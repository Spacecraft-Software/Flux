// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};

pub struct SystemdResolvedBackend;

impl DnsBackend for SystemdResolvedBackend {
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<(), AppError> {
        let tier = tier.unwrap_or(Tier::Standard);
        let addrs = provider
            .addresses
            .get(&tier)
            .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

        let mut dns_lines = Vec::new();
        if let Some(ip) = &addrs.ipv4_primary {
            if !ipv6_only {
                dns_lines.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv4_secondary {
            if !ipv6_only {
                dns_lines.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv6_primary {
            if !ipv4_only {
                dns_lines.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv6_secondary {
            if !ipv4_only {
                dns_lines.push(ip.clone());
            }
        }

        let dns_value = dns_lines.join(" ");

        let mut conf = format!("# flux-managed\n[Resolve]\nDNS={dns_value}\n");

        match protocol {
            Protocol::DoT => {
                conf.push_str("DNSOverTLS=yes\n");
            }
            Protocol::DoH => {
                if let Some(url) = &addrs.doh_url {
                    conf.push_str(&format!("DNSOverHTTPS={url}\n"));
                }
            }
            Protocol::Plain => {}
            _ => {
                return Err(AppError::apply_failed(format!(
                    "systemd-resolved does not support protocol {protocol}"
                )));
            }
        }

        conf.push_str("DNSSEC=yes\n");

        crate::privilege::create_dir_all("/etc/systemd/resolved.conf.d")
            .map_err(|e| AppError::apply_failed(format!("Failed to create conf.d: {e}")))?;

        crate::privilege::write_file("/etc/systemd/resolved.conf.d/flux.conf", &conf)
            .map_err(|e| AppError::apply_failed(format!("Failed to write resolved.conf: {e}")))?;

        // Restart systemd-resolved
        crate::privilege::run_command("systemctl", &["restart", "systemd-resolved"]).map_err(
            |e| AppError::apply_failed(format!("Failed to restart systemd-resolved: {e}")),
        )?;

        Ok(())
    }

    fn backup(&self) -> Result<BackupRecord, AppError> {
        let path = std::path::Path::new("/etc/systemd/resolved.conf.d/flux.conf");
        let content = if path.exists() {
            std::fs::read_to_string(path).unwrap_or_default()
        } else {
            "# no flux config present\n".to_string()
        };
        Ok(BackupRecord::new(Backend::SystemdResolved, content))
    }

    fn restore(&self, record: &BackupRecord) -> Result<(), AppError> {
        if record.snapshot.trim() == "# no flux config present" {
            let _ = crate::privilege::remove_file("/etc/systemd/resolved.conf.d/flux.conf");
        } else {
            crate::privilege::write_file(
                "/etc/systemd/resolved.conf.d/flux.conf",
                &record.snapshot,
            )
            .map_err(|e| AppError::apply_failed(format!("Failed to restore: {e}")))?;
        }
        crate::privilege::run_command("systemctl", &["restart", "systemd-resolved"]).map_err(
            |e| AppError::apply_failed(format!("Failed to restart systemd-resolved: {e}")),
        )?;
        Ok(())
    }

    fn status(&self) -> Result<BackendStatus, AppError> {
        // Read resolvectl status
        let output = std::process::Command::new("resolvectl")
            .args(["status"])
            .output()
            .ok();

        let mut nameservers = Vec::new();
        let mut dot = false;
        let mut doh = false;

        if let Some(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("DNS Servers:") {
                    if let Some(ns) = line.split(':').nth(1) {
                        nameservers.push(ns.trim().to_string());
                    }
                }
                if line.contains("DNSOverTLS") && line.contains("yes") {
                    dot = true;
                }
                if line.contains("DNSOverHTTPS") {
                    doh = true;
                }
            }
        }

        Ok(BackendStatus {
            backend: "systemd-resolved".to_string(),
            active: !nameservers.is_empty(),
            nameservers: Some(nameservers),
            dot_enabled: Some(dot),
            doh_enabled: Some(doh),
        })
    }

    fn verify(&self, _timeout_secs: u64) -> Result<VerifyResult, AppError> {
        let start = std::time::Instant::now();
        let output = std::process::Command::new("resolvectl")
            .args(["query", "--cache=no", "dns.google"])
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
