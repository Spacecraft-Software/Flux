// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};

macro_rules! bsd_backend {
    ($struct_name:ident, $backend_variant:path, $backend_str:expr) => {
        pub struct $struct_name;

        impl DnsBackend for $struct_name {
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
                        "{0} backend only supports plain DNS in v0.1.0; requested {protocol}",
                        $backend_str
                    )));
                }

                let tier = tier.unwrap_or(Tier::Standard);
                let addrs = provider
                    .addresses
                    .get(&tier)
                    .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

                let mut lines = vec![format!("# flux-managed ({0})", $backend_str)];
                if let Some(ip) = &addrs.ipv4_primary {
                    if !ipv6_only {
                        lines.push(format!("nameserver {ip}"));
                    }
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
                crate::privilege::write_file("/etc/resolv.conf", &content).map_err(|e| {
                    AppError::apply_failed(format!("Failed to write resolv.conf: {e}"))
                })?;

                Ok(())
            }

            fn backup(&self) -> Result<BackupRecord, AppError> {
                let content = std::fs::read_to_string("/etc/resolv.conf").map_err(|e| {
                    AppError::apply_failed(format!("Failed to read resolv.conf: {e}"))
                })?;
                Ok(BackupRecord::new($backend_variant, content))
            }

            fn restore(&self, record: &BackupRecord) -> Result<(), AppError> {
                crate::privilege::write_file("/etc/resolv.conf", &record.snapshot).map_err(
                    |e| AppError::apply_failed(format!("Failed to restore resolv.conf: {e}")),
                )?;
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
                    backend: $backend_str.to_string(),
                    active: !nameservers.is_empty(),
                    nameservers: Some(nameservers),
                    dot_enabled: Some(false),
                    doh_enabled: Some(false),
                })
            }

            fn verify(&self, timeout_secs: u64) -> Result<VerifyResult, AppError> {
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
    };
}

bsd_backend!(FreeBsdBackend, Backend::FreeBsd, "freebsd");
bsd_backend!(OpenBsdBackend, Backend::OpenBsd, "openbsd");
bsd_backend!(NetBsdBackend, Backend::NetBsd, "netbsd");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freebsd_backend_status_empty() {
        let backend = FreeBsdBackend;
        let status = backend.status().unwrap();
        assert_eq!(status.backend, "freebsd");
        // On Linux test systems /etc/resolv.conf usually exists
        // so active may be true or false depending on content.
        assert!(status.nameservers.is_some());
    }

    #[test]
    fn test_openbsd_backend_status_empty() {
        let backend = OpenBsdBackend;
        let status = backend.status().unwrap();
        assert_eq!(status.backend, "openbsd");
        assert!(status.nameservers.is_some());
    }

    #[test]
    fn test_netbsd_backend_status_empty() {
        let backend = NetBsdBackend;
        let status = backend.status().unwrap();
        assert_eq!(status.backend, "netbsd");
        assert!(status.nameservers.is_some());
    }

    #[test]
    fn test_bsd_apply_rejects_dot() {
        let backend = FreeBsdBackend;
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let result = backend.apply(provider, None, Protocol::DoT, false, false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("only supports plain DNS"));
    }

    #[test]
    fn test_bsd_apply_rejects_doh() {
        let backend = OpenBsdBackend;
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let result = backend.apply(provider, None, Protocol::DoH, false, false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("only supports plain DNS"));
    }
}
