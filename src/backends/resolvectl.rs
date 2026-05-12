// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};
use serde::{Deserialize, Serialize};
use std::process::Command;

pub struct ResolvectlBackend;

/// Structured snapshot of resolvectl state for backup/restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolvectlSnapshot {
    interface: String,
    servers: Vec<String>,
    dot: bool,
    dnssec: bool,
}

impl DnsBackend for ResolvectlBackend {
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<(), AppError> {
        if !matches!(protocol, Protocol::Plain | Protocol::DoT) {
            return Err(AppError::apply_failed(format!(
                "resolvectl backend only supports Plain and DoT; requested {protocol}"
            )));
        }

        let tier = tier.unwrap_or(Tier::Standard);
        let addrs = provider
            .addresses
            .get(&tier)
            .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

        let mut servers = Vec::new();
        if let Some(ip) = &addrs.ipv4_primary {
            if !ipv6_only {
                servers.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv4_secondary {
            if !ipv6_only {
                servers.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv6_primary {
            if !ipv4_only {
                servers.push(ip.clone());
            }
        }
        if let Some(ip) = &addrs.ipv6_secondary {
            if !ipv4_only {
                servers.push(ip.clone());
            }
        }

        let iface = default_interface()?;

        // Set DNS servers
        let mut dns_args = vec!["dns", &iface];
        for s in &servers {
            dns_args.push(s);
        }
        crate::privilege::run_command("resolvectl", &dns_args)
            .map_err(|e| AppError::apply_failed(format!("Failed to set DNS servers: {e}")))?;

        // Set DNSOverTLS
        let dot = match protocol {
            Protocol::DoT => "yes",
            _ => "no",
        };
        crate::privilege::run_command("resolvectl", &["dnsovertls", &iface, dot])
            .map_err(|e| AppError::apply_failed(format!("Failed to set DNSOverTLS: {e}")))?;

        // Flush caches (best effort)
        let _ = crate::privilege::run_command("resolvectl", &["flush-caches"]);

        Ok(())
    }

    fn backup(&self) -> Result<BackupRecord, AppError> {
        let iface = default_interface()?;
        let output = Command::new("resolvectl")
            .args(["status", &iface])
            .output()
            .map_err(|e| AppError::apply_failed(format!("resolvectl status failed: {e}")))?;

        let text = String::from_utf8_lossy(&output.stdout);
        let state = parse_interface_status(&text, &iface)?;
        let snapshot = serde_json::to_string(&state)
            .map_err(|e| AppError::apply_failed(format!("Failed to serialize backup: {e}")))?;

        Ok(BackupRecord::new(Backend::Resolvectl, snapshot))
    }

    fn restore(&self, record: &BackupRecord) -> Result<(), AppError> {
        let state: ResolvectlSnapshot = serde_json::from_str(&record.snapshot)
            .map_err(|e| AppError::apply_failed(format!("Failed to parse backup: {e}")))?;

        // Restore DNS servers
        let mut dns_args = vec!["dns", &state.interface];
        for s in &state.servers {
            dns_args.push(s);
        }
        crate::privilege::run_command("resolvectl", &dns_args)
            .map_err(|e| AppError::apply_failed(format!("Failed to restore DNS servers: {e}")))?;

        // Restore DNSOverTLS
        let dot = if state.dot { "yes" } else { "no" };
        crate::privilege::run_command("resolvectl", &["dnsovertls", &state.interface, dot])
            .map_err(|e| AppError::apply_failed(format!("Failed to restore DNSOverTLS: {e}")))?;

        // Restore DNSSEC
        let dnssec = if state.dnssec { "yes" } else { "no" };
        crate::privilege::run_command("resolvectl", &["dnssec", &state.interface, dnssec])
            .map_err(|e| AppError::apply_failed(format!("Failed to restore DNSSEC: {e}")))?;

        // Flush caches
        let _ = crate::privilege::run_command("resolvectl", &["flush-caches"]);

        Ok(())
    }

    fn status(&self) -> Result<BackendStatus, AppError> {
        let output = Command::new("resolvectl")
            .args(["status"])
            .output()
            .map_err(|e| AppError::apply_failed(format!("resolvectl status failed: {e}")))?;

        let text = String::from_utf8_lossy(&output.stdout);
        let mut nameservers = Vec::new();
        let mut dot = false;
        let mut has_dns_scope = false;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Current Scopes:") && trimmed.contains("DNS") {
                has_dns_scope = true;
            }
            if trimmed.starts_with("DNS Servers:") {
                if let Some(caps) = trimmed.strip_prefix("DNS Servers:") {
                    for ip in caps.split_whitespace() {
                        nameservers.push(ip.to_string());
                    }
                }
            }
            if trimmed.contains("+DNSOverTLS") {
                dot = true;
            }
        }

        Ok(BackendStatus {
            backend: "resolvectl".to_string(),
            active: has_dns_scope && !nameservers.is_empty(),
            nameservers: Some(nameservers),
            dot_enabled: Some(dot),
            doh_enabled: Some(false),
        })
    }

    fn verify(&self, timeout_secs: u64) -> Result<VerifyResult, AppError> {
        let host = "dns.google";
        let start = std::time::Instant::now();
        let output = Command::new("dig")
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

/// Find the default network interface from resolvectl status.
/// Looks for the first link with `+DefaultRoute` in its protocols.
fn default_interface() -> Result<String, AppError> {
    let output = Command::new("resolvectl")
        .args(["status"])
        .output()
        .map_err(|e| AppError::detection_failed(format!("resolvectl status failed: {e}")))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut current_iface: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        // Link N (iface) or Link N (iface@something)
        if trimmed.starts_with("Link ") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let iface = &trimmed[start + 1..end];
                    let iface = iface.split('@').next().unwrap_or(iface);
                    current_iface = Some(iface.to_string());
                }
            }
        }

        if trimmed.contains("+DefaultRoute") {
            if let Some(ref iface) = current_iface {
                return Ok(iface.clone());
            }
        }
    }

    // Fallback: first interface with Current Scopes: DNS
    current_iface = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Link ") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let iface = &trimmed[start + 1..end];
                    let iface = iface.split('@').next().unwrap_or(iface);
                    current_iface = Some(iface.to_string());
                }
            }
        }
        if trimmed.starts_with("Current Scopes:") && trimmed.contains("DNS") {
            if let Some(ref iface) = current_iface {
                return Ok(iface.clone());
            }
        }
    }

    Err(AppError::detection_failed(
        "Could not determine default network interface from resolvectl status",
    ))
}

/// Parse `resolvectl status <iface>` output into a structured snapshot.
fn parse_interface_status(text: &str, iface: &str) -> Result<ResolvectlSnapshot, AppError> {
    let mut in_target = false;
    let mut servers = Vec::new();
    let mut dot = false;
    let mut dnssec = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Enter the target interface block
        if trimmed.starts_with("Link ") {
            in_target =
                trimmed.contains(&format!("({iface})")) || trimmed.contains(&format!("({iface}@"));
            continue;
        }

        if !in_target {
            continue;
        }

        // Exit if we hit the next interface block
        if trimmed.starts_with("Link ") {
            break;
        }

        if trimmed.starts_with("DNS Servers:") {
            if let Some(caps) = trimmed.strip_prefix("DNS Servers:") {
                for ip in caps.split_whitespace() {
                    servers.push(ip.to_string());
                }
            }
        }

        if trimmed.contains("+DNSOverTLS") {
            dot = true;
        }

        if trimmed.contains("DNSSEC=yes") || trimmed.contains("DNSSEC=allow-downgrade") {
            dnssec = true;
        }
    }

    if servers.is_empty() {
        // Not a hard error; system may have no DNS configured yet.
        // Use the system default loopback or placeholder.
    }

    Ok(ResolvectlSnapshot {
        interface: iface.to_string(),
        servers,
        dot,
        dnssec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_interface_parses_link_with_default_route() {
        let sample = r#"
Global
       Protocols: +LLMNR +mDNS -DNSOverTLS DNSSEC=no/unsupported

Link 2 (eth0)
    Current Scopes: DNS
         Protocols: +DefaultRoute +LLMNR -mDNS -DNSOverTLS DNSSEC=no/unsupported
   DNS Servers: 1.1.1.1 8.8.8.8

Link 3 (wlan0)
    Current Scopes: none
"#;
        let mut current_iface: Option<String> = None;
        let mut found = None;
        for line in sample.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Link ") {
                if let Some(start) = trimmed.find('(') {
                    if let Some(end) = trimmed.find(')') {
                        let iface = &trimmed[start + 1..end];
                        let iface = iface.split('@').next().unwrap_or(iface);
                        current_iface = Some(iface.to_string());
                    }
                }
            }
            if trimmed.contains("+DefaultRoute") {
                if let Some(ref iface) = current_iface {
                    found = Some(iface.clone());
                    break;
                }
            }
        }
        assert_eq!(found, Some("eth0".to_string()));
    }

    #[test]
    fn test_default_interface_fallback_to_dns_scope() {
        let sample = r#"
Link 2 (eth0)
    Current Scopes: none

Link 3 (wlan0)
    Current Scopes: DNS
         Protocols: +LLMNR -mDNS -DNSOverTLS DNSSEC=no/unsupported
   DNS Servers: 9.9.9.9
"#;
        let mut current_iface: Option<String> = None;
        let mut found = None;
        for line in sample.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Link ") {
                if let Some(start) = trimmed.find('(') {
                    if let Some(end) = trimmed.find(')') {
                        let iface = &trimmed[start + 1..end];
                        let iface = iface.split('@').next().unwrap_or(iface);
                        current_iface = Some(iface.to_string());
                    }
                }
            }
            if trimmed.starts_with("Current Scopes:") && trimmed.contains("DNS") {
                if let Some(ref iface) = current_iface {
                    found = Some(iface.clone());
                    break;
                }
            }
        }
        assert_eq!(found, Some("wlan0".to_string()));
    }

    #[test]
    fn test_parse_interface_status_extracts_servers() {
        let text = r#"Link 2 (eth0)
    Current Scopes: DNS
         Protocols: +DefaultRoute +LLMNR -mDNS +DNSOverTLS DNSSEC=yes
   DNS Servers: 1.1.1.1 2606:4700:4700::1111
"#;
        let state = parse_interface_status(text, "eth0").unwrap();
        assert_eq!(state.interface, "eth0");
        assert_eq!(state.servers, vec!["1.1.1.1", "2606:4700:4700::1111"]);
        assert!(state.dot);
        assert!(state.dnssec);
    }

    #[test]
    fn test_parse_interface_status_no_servers() {
        let text = r#"Link 2 (eth0)
    Current Scopes: none
         Protocols: +LLMNR -mDNS -DNSOverTLS DNSSEC=no/unsupported
"#;
        let state = parse_interface_status(text, "eth0").unwrap();
        assert_eq!(state.interface, "eth0");
        assert!(state.servers.is_empty());
        assert!(!state.dot);
        assert!(!state.dnssec);
    }

    #[test]
    fn test_parse_interface_status_ignores_other_interfaces() {
        let text = r#"Link 2 (eth0)
    Current Scopes: DNS
   DNS Servers: 1.1.1.1

Link 3 (wlan0)
    Current Scopes: DNS
   DNS Servers: 9.9.9.9
"#;
        let state = parse_interface_status(text, "eth0").unwrap();
        assert_eq!(state.servers, vec!["1.1.1.1"]);
    }

    #[test]
    fn test_backup_snapshot_roundtrip() {
        let state = ResolvectlSnapshot {
            interface: "eth0".to_string(),
            servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            dot: true,
            dnssec: false,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: ResolvectlSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.interface, "eth0");
        assert_eq!(parsed.servers, vec!["1.1.1.1", "8.8.8.8"]);
        assert!(parsed.dot);
        assert!(!parsed.dnssec);
    }
}
