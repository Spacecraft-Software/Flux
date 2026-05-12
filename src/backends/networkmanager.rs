// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};

pub struct NetworkManagerBackend;

impl DnsBackend for NetworkManagerBackend {
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<(), AppError> {
        if protocol != Protocol::Plain {
            return Err(AppError::apply_failed(
                "NetworkManager backend only supports plain DNS",
            ));
        }

        let tier = tier.unwrap_or(Tier::Standard);
        let addrs = provider
            .addresses
            .get(&tier)
            .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

        let mut dns4 = Vec::new();
        let mut dns6 = Vec::new();

        if let Some(ip) = &addrs.ipv4_primary {
            if !ipv6_only {
                dns4.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv4_secondary {
            if !ipv6_only {
                dns4.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv6_primary {
            if !ipv4_only {
                dns6.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv6_secondary {
            if !ipv4_only {
                dns6.push(ip.clone());
            }
        }

        // Get active connection
        let active = std::process::Command::new("nmcli")
            .args(["-t", "-f", "NAME,DEVICE", "connection", "show", "--active"])
            .output()
            .map_err(|e| AppError::apply_failed(format!("nmcli failed: {e}")))?;

        let stdout = String::from_utf8_lossy(&active.stdout);
        let first_line = stdout.lines().next().unwrap_or("");
        let con_name = first_line.split(':').next().unwrap_or("").trim();

        if con_name.is_empty() {
            return Err(AppError::apply_failed(
                "No active NetworkManager connection found",
            ));
        }

        if !dns4.is_empty() {
            let _ = std::process::Command::new("nmcli")
                .args([
                    "connection",
                    "modify",
                    con_name,
                    "ipv4.dns",
                    &dns4.join(","),
                ])
                .status();
        }
        if !dns6.is_empty() {
            let _ = std::process::Command::new("nmcli")
                .args([
                    "connection",
                    "modify",
                    con_name,
                    "ipv6.dns",
                    &dns6.join(","),
                ])
                .status();
        }

        // Re-activate connection
        let _ = std::process::Command::new("nmcli")
            .args(["connection", "up", con_name])
            .status();

        Ok(())
    }

    fn backup(&self) -> Result<BackupRecord, AppError> {
        let output = std::process::Command::new("nmcli")
            .args(["connection", "show", "--active"])
            .output()
            .ok();

        let snapshot = output
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        Ok(BackupRecord::new(Backend::NetworkManager, snapshot))
    }

    fn restore(&self, _record: &BackupRecord) -> Result<(), AppError> {
        // NetworkManager backup/restore is complex; for MVP we note the limitation
        Err(AppError::apply_failed(
            "NetworkManager restore not fully implemented in v0.1.0",
        ))
    }

    fn status(&self) -> Result<BackendStatus, AppError> {
        let output = std::process::Command::new("nmcli")
            .args(["device", "show"])
            .output()
            .ok();

        let mut nameservers = Vec::new();
        if let Some(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("IP4.DNS") || line.contains("IP6.DNS") {
                    if let Some(ns) = line.split_whitespace().last() {
                        nameservers.push(ns.to_string());
                    }
                }
            }
        }

        Ok(BackendStatus {
            backend: "networkmanager".to_string(),
            active: !nameservers.is_empty(),
            nameservers: Some(nameservers),
            dot_enabled: Some(false),
            doh_enabled: Some(false),
        })
    }

    fn verify(&self, timeout_secs: u64) -> Result<VerifyResult, AppError> {
        let start = std::time::Instant::now();
        let output = std::process::Command::new("dig")
            .args(["+short", "+time", &timeout_secs.to_string(), "dns.google"])
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
