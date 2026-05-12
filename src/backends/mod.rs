// SPDX-License-Identifier: GPL-3.0-or-later

use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};
use serde::{Deserialize, Serialize};

pub mod bsd;
pub mod networkmanager;
pub mod nixos;
pub mod resolvconf;
pub mod resolvectl;
pub mod stub;
pub mod systemd;

/// Common interface for DNS backend adapters.
pub trait DnsBackend {
    /// Apply DNS configuration for the given provider/tier/protocol.
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<(), AppError>;

    /// Create a backup of current DNS state.
    fn backup(&self) -> Result<BackupRecord, AppError>;

    /// Restore from a backup record.
    #[allow(dead_code)]
    fn restore(&self, record: &BackupRecord) -> Result<(), AppError>;

    /// Get current backend status.
    fn status(&self) -> Result<BackendStatus, AppError>;

    /// Verify DNS resolution is working.
    fn verify(&self, timeout_secs: u64) -> Result<VerifyResult, AppError>;
}

/// A backup record with UTC timestamp and snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub timestamp: String,
    pub backend: String,
    pub snapshot: String,
}

impl BackupRecord {
    pub fn new(backend: Backend, snapshot: String) -> Self {
        Self {
            timestamp: crate::output::now_utc(),
            backend: backend.to_string(),
            snapshot,
        }
    }
}

/// Backend status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub backend: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dot_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doh_enabled: Option<bool>,
}

/// Verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver_ip: Option<String>,
    /// Round-trip time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtt_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_record_timestamp_format() {
        let record = BackupRecord::new(Backend::ResolvConf, "nameserver 1.1.1.1".to_string());
        // Must be ISO 8601 UTC with Z suffix: YYYY-MM-DDTHH:MM:SSZ
        assert!(record.timestamp.ends_with('Z'), "Timestamp must end with Z");
        assert!(
            record.timestamp.contains('T'),
            "Timestamp must contain T separator"
        );
        // Rough length check for YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(record.timestamp.len(), 20, "Timestamp should be 20 chars");
    }

    #[test]
    fn test_backup_record_serde() {
        let record = BackupRecord::new(Backend::SystemdResolved, "DNS=1.1.1.1".to_string());
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("systemd-resolved"));
        let decoded: BackupRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.backend, record.backend);
        assert_eq!(decoded.snapshot, record.snapshot);
    }

    #[test]
    fn test_verify_result_serde() {
        let result = VerifyResult {
            success: true,
            resolver_ip: Some("1.1.1.1".to_string()),
            rtt_ms: Some(42),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        // Null fields omitted
        assert!(!json.contains("error"));
        assert!(json.contains("rtt_ms"));
        let decoded: VerifyResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.rtt_ms, Some(42));
    }
}
