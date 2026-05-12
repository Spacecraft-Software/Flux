// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backends::{BackupRecord, DnsBackend};
use crate::detection::Backend;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Backup metadata stored alongside each snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub timestamp: String,
    pub schema_version: String,
}

/// Manage backups under `~/.local/share/flux/backups/`.
pub struct BackupManager {
    backup_dir: PathBuf,
    keep_max: usize,
}

impl BackupManager {
    pub fn new() -> Result<Self, AppError> {
        let backup_dir = dirs::home_dir()
            .ok_or_else(|| AppError::apply_failed("Cannot determine home directory"))?
            .join(".local/share/flux/backups");

        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| AppError::apply_failed(format!("Failed to create backup dir: {e}")))?;

        Ok(Self {
            backup_dir,
            keep_max: 10,
        })
    }

    #[allow(dead_code)]
    pub fn backup_dir(&self) -> &std::path::Path {
        &self.backup_dir
    }

    /// Save a backup record and prune old backups.
    pub fn save(&self, record: &BackupRecord, provider: Option<&str>) -> Result<PathBuf, AppError> {
        let filename = format!("{}_{}.bak", record.timestamp, record.backend);
        let path = self.backup_dir.join(&filename);

        let payload = BackupPayload {
            record: record.clone(),
            metadata: BackupMetadata {
                backend: record.backend.clone(),
                provider: provider.map(|s| s.to_string()),
                timestamp: record.timestamp.clone(),
                schema_version: "0.1.0".to_string(),
            },
        };

        let json = serde_json::to_string_pretty(&payload)
            .map_err(|e| AppError::apply_failed(format!("Failed to serialize backup: {e}")))?;

        std::fs::write(&path, json)
            .map_err(|e| AppError::apply_failed(format!("Failed to write backup: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)
                .map_err(|e| AppError::apply_failed(format!("Failed to get metadata: {e}")))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms)
                .map_err(|e| AppError::apply_failed(format!("Failed to chmod backup: {e}")))?;
        }

        self.prune()?;
        Ok(path)
    }

    /// List backups from newest to oldest.
    pub fn list(&self) -> Result<Vec<PathBuf>, AppError> {
        let mut entries: Vec<_> = std::fs::read_dir(&self.backup_dir)
            .map_err(|e| AppError::apply_failed(format!("Failed to read backup dir: {e}")))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.extension().is_some_and(|ext| ext == "bak")
                    && e.metadata().map(|m| m.len() > 0).unwrap_or(false)
            })
            .map(|e| (e.path(), e.metadata().and_then(|m| m.modified()).ok()))
            .collect();

        entries.sort_by(|a, b| match (b.1, a.1) {
            (Some(tb), Some(ta)) => tb.cmp(&ta),
            _ => b.0.cmp(&a.0),
        });

        Ok(entries.into_iter().map(|(p, _)| p).collect())
    }

    /// Get the most recent backup for a given backend.
    pub fn latest(&self, backend: Backend) -> Result<Option<BackupPayload>, AppError> {
        let suffix = format!("_{backend}.bak");
        let mut entries: Vec<_> = self
            .list()?
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(&suffix))
            })
            .collect();

        entries.sort_by(|a, b| b.cmp(a));

        if let Some(path) = entries.first() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| AppError::apply_failed(format!("Failed to read backup: {e}")))?;
            let payload: BackupPayload = serde_json::from_str(&content)
                .map_err(|e| AppError::apply_failed(format!("Failed to parse backup: {e}")))?;
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }

    /// Restore from the most recent backup for a backend.
    pub fn restore(&self, backend: Backend) -> Result<(), AppError> {
        let payload = self
            .latest(backend)?
            .ok_or_else(|| AppError::apply_failed("No backup found for this backend"))?;

        match backend {
            Backend::ResolvConf => {
                let b = crate::backends::resolvconf::ResolvConfBackend;
                b.restore(&payload.record)?;
            }
            Backend::SystemdResolved => {
                let b = crate::backends::systemd::SystemdResolvedBackend;
                b.restore(&payload.record)?;
            }
            Backend::NetworkManager => {
                let b = crate::backends::networkmanager::NetworkManagerBackend;
                b.restore(&payload.record)?;
            }
            Backend::Nixos => {
                let b = crate::backends::nixos::NixosBackend;
                b.restore(&payload.record)?;
            }
            Backend::Resolvectl => {
                let b = crate::backends::resolvectl::ResolvectlBackend;
                b.restore(&payload.record)?;
            }
            Backend::FreeBsd => {
                let b = crate::backends::bsd::FreeBsdBackend;
                b.restore(&payload.record)?;
            }
            Backend::OpenBsd => {
                let b = crate::backends::bsd::OpenBsdBackend;
                b.restore(&payload.record)?;
            }
            Backend::NetBsd => {
                let b = crate::backends::bsd::NetBsdBackend;
                b.restore(&payload.record)?;
            }
            Backend::Stub => {
                let b = crate::backends::stub::StubBackend;
                b.restore(&payload.record)?;
            }
        }

        Ok(())
    }

    /// Prune old backups, keeping the most recent N.
    fn prune(&self) -> Result<(), AppError> {
        let mut entries = self.list()?;
        if entries.len() > self.keep_max {
            entries.reverse(); // oldest first
            let to_remove = entries.len() - self.keep_max;
            for path in entries.iter().take(to_remove) {
                let _ = std::fs::remove_file(path);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    pub record: BackupRecord,
    pub metadata: BackupMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_manager_new() {
        let mgr = BackupManager::new();
        assert!(mgr.is_ok());
    }

    #[test]
    fn test_backup_save_and_list() {
        let mgr = BackupManager::new().unwrap();
        let record = BackupRecord::new(Backend::ResolvConf, "nameserver 1.1.1.1".to_string());
        let path = mgr.save(&record, Some("cloudflare")).unwrap();
        assert!(path.exists());

        let list = mgr.list().unwrap();
        assert!(!list.is_empty());
    }

    #[test]
    fn test_backup_latest() {
        let mgr = BackupManager::new().unwrap();
        let record = BackupRecord::new(Backend::ResolvConf, "nameserver 1.1.1.1".to_string());
        mgr.save(&record, Some("cloudflare")).unwrap();

        let latest = mgr.latest(Backend::ResolvConf).unwrap();
        assert!(latest.is_some());
        assert_eq!(
            latest.unwrap().metadata.provider,
            Some("cloudflare".to_string())
        );
    }
}
