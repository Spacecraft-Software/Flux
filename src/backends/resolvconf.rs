// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};

pub struct ResolvConfBackend;

impl DnsBackend for ResolvConfBackend {
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<(), AppError> {
        if protocol != Protocol::Plain {
            return Err(AppError::apply_failed(format!(
                "resolv.conf backend only supports plain DNS; requested {protocol}"
            )));
        }

        let tier = tier.unwrap_or(Tier::Standard);
        let addrs = provider
            .addresses
            .get(&tier)
            .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

        let mut lines = vec!["# flux-managed".to_string()];
        if let Some(ip) = &addrs.ipv4_primary {
            lines.push(format!("nameserver {ip}"));
        }
        if let Some(ip) = &addrs.ipv4_secondary {
            if !ipv6_only {
                lines.push(format!("nameserver {ip}"));
            }
        }
        if let Some(ip) = &addrs.ipv6_primary {
            if !ipv4_only {
                lines.push(format!("nameserver {ip}"));
            }
        }
        if let Some(ip) = &addrs.ipv6_secondary {
            if !ipv4_only {
                lines.push(format!("nameserver {ip}"));
            }
        }

        let content = lines.join("\n") + "\n";
        crate::privilege::write_file("/etc/resolv.conf", &content)
            .map_err(|e| AppError::apply_failed(format!("Failed to write resolv.conf: {e}")))?;

        Ok(())
    }

    fn backup(&self) -> Result<BackupRecord, AppError> {
        let content = std::fs::read_to_string("/etc/resolv.conf")
            .map_err(|e| AppError::apply_failed(format!("Failed to read resolv.conf: {e}")))?;
        Ok(BackupRecord::new(Backend::ResolvConf, content))
    }

    fn restore(&self, record: &BackupRecord) -> Result<(), AppError> {
        crate::privilege::write_file("/etc/resolv.conf", &record.snapshot)
            .map_err(|e| AppError::apply_failed(format!("Failed to restore resolv.conf: {e}")))?;
        Ok(())
    }

    fn status(&self) -> Result<BackendStatus, AppError> {
        let content = std::fs::read_to_string("/etc/resolv.conf").unwrap_or_default();
        let nameservers: Vec<String> = content
            .lines()
            .filter(|l| l.trim_start().starts_with("nameserver"))
            .map(|l| l.split_whitespace().nth(1).unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(BackendStatus {
            backend: "resolv.conf".to_string(),
            active: !nameservers.is_empty(),
            nameservers: Some(nameservers),
            dot_enabled: Some(false),
            doh_enabled: Some(false),
        })
    }

    fn verify(&self, timeout_secs: u64) -> Result<VerifyResult, AppError> {
        // Use dig or host to verify
        let host = "dns.google";
        let start = std::time::Instant::now();
        let output = std::process::Command::new("dig")
            .args(["+short", "+time", &timeout_secs.to_string(), host])
            .output();

        match output {
            Ok(out) if out.status.success() => Ok(VerifyResult {
                success: true,
                resolver_ip: None,
                rtt_ms: Some(start.elapsed().as_millis() as u64),
                error: None,
            }),
            Ok(out) => Ok(VerifyResult {
                success: false,
                resolver_ip: None,
                rtt_ms: None,
                error: Some(String::from_utf8_lossy(&out.stderr).to_string()),
            }),
            Err(e) => Ok(VerifyResult {
                success: false,
                resolver_ip: None,
                rtt_ms: None,
                error: Some(e.to_string()),
            }),
        }
    }
}
