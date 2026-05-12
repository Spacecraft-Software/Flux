// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};
use std::io::Write;

pub struct NixosBackend;

impl NixosBackend {
    /// Generate the Nix expression fragment without writing to stdout.
    /// Exposed for unit testing.
    pub fn generate_expression(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<String, AppError> {
        let tier = tier.unwrap_or(Tier::Standard);
        let addrs = provider
            .addresses
            .get(&tier)
            .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

        let mut nameservers = Vec::new();
        if let Some(ip) = &addrs.ipv4_primary {
            if !ipv6_only {
                nameservers.push(format!("\"{ip}\""));
            }
        }
        if let Some(ip) = &addrs.ipv4_secondary {
            if !ipv6_only {
                nameservers.push(format!("\"{ip}\""));
            }
        }
        if let Some(ip) = &addrs.ipv6_primary {
            if !ipv4_only {
                nameservers.push(format!("\"{ip}\""));
            }
        }
        if let Some(ip) = &addrs.ipv6_secondary {
            if !ipv4_only {
                nameservers.push(format!("\"{ip}\""));
            }
        }

        let ns_list = nameservers.join(" ");

        let mut expr = format!(
            "# Flux-generated DNS configuration for {} {} {}\n",
            provider.name, tier, protocol
        );
        expr.push_str(&format!("networking.nameservers = [ {ns_list} ];\n"));

        match protocol {
            Protocol::DoT => {
                expr.push_str("services.resolved = {\n");
                expr.push_str("  enable = true;\n");
                expr.push_str("  dnssec = \"true\";\n");
                expr.push_str("  dnsovertls = \"true\";  # for DoT\n");
                expr.push_str("};\n");
            }
            Protocol::DoH => {
                expr.push_str("services.resolved = {\n");
                expr.push_str("  enable = true;\n");
                expr.push_str("  dnssec = \"true\";\n");
                expr.push_str("  # DoH: add DNSOverHTTPS= to extraConfig\n");
                expr.push_str("};\n");
            }
            Protocol::Plain => {}
            _ => {
                return Err(AppError::apply_failed(format!(
                    "NixOS backend does not support protocol {protocol}"
                )));
            }
        }

        Ok(expr)
    }
}

impl DnsBackend for NixosBackend {
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        ipv4_only: bool,
        ipv6_only: bool,
    ) -> Result<(), AppError> {
        let expr = self.generate_expression(provider, tier, protocol, ipv4_only, ipv6_only)?;

        std::io::stdout()
            .write_all(expr.as_bytes())
            .map_err(|e| AppError::apply_failed(format!("Failed to write Nix expression: {e}")))?;

        Ok(())
    }

    fn backup(&self) -> Result<BackupRecord, AppError> {
        Ok(BackupRecord::new(
            Backend::Nixos,
            "# NixOS backups are manual — save your configuration.nix before changes".to_string(),
        ))
    }

    fn restore(&self, _record: &BackupRecord) -> Result<(), AppError> {
        Err(AppError::apply_failed(
            "NixOS restore requires manual nixos-rebuild",
        ))
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
            backend: "nixos".to_string(),
            active: !nameservers.is_empty(),
            nameservers: Some(nameservers),
            dot_enabled: None,
            doh_enabled: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{Protocol, Tier};

    #[test]
    fn test_nixos_expression_contains_nameservers() {
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let backend = NixosBackend;
        let expr = backend
            .generate_expression(
                provider,
                Some(Tier::Standard),
                Protocol::Plain,
                false,
                false,
            )
            .unwrap();

        assert!(
            expr.contains("networking.nameservers"),
            "Expression must contain networking.nameservers"
        );
        assert!(
            expr.contains("1.1.1.1"),
            "Expression must contain provider primary IPv4"
        );
    }

    #[test]
    fn test_nixos_expression_dot() {
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let backend = NixosBackend;
        let expr = backend
            .generate_expression(provider, Some(Tier::Standard), Protocol::DoT, false, false)
            .unwrap();

        assert!(expr.contains("dnsovertls = \"true\""));
        assert!(expr.contains("services.resolved"));
    }

    #[test]
    fn test_nixos_expression_doh() {
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let backend = NixosBackend;
        let expr = backend
            .generate_expression(provider, Some(Tier::Standard), Protocol::DoH, false, false)
            .unwrap();

        assert!(expr.contains("services.resolved"));
        assert!(expr.contains("DoH"));
    }

    #[test]
    fn test_nixos_backup_is_manual() {
        let backend = NixosBackend;
        let record = backend.backup().unwrap();
        assert_eq!(record.backend, "nixos");
        assert!(record.snapshot.contains("manual"));
    }

    #[test]
    fn test_nixos_restore_fails() {
        let backend = NixosBackend;
        let record = BackupRecord::new(Backend::Nixos, "test".to_string());
        let result = backend.restore(&record);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("nixos-rebuild"));
    }

    #[test]
    fn test_nixos_status_declarative() {
        let backend = NixosBackend;
        let status = backend.status().unwrap();
        assert_eq!(status.backend, "nixos");
    }
}
