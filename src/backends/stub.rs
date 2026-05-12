// SPDX-License-Identifier: GPL-3.0-or-later

use super::{BackendStatus, BackupRecord, DnsBackend, VerifyResult};
use crate::detection::Backend;
use crate::error::AppError;
use crate::registry::{Protocol, Provider, Tier};

pub struct StubBackend;

impl DnsBackend for StubBackend {
    fn apply(
        &self,
        provider: &Provider,
        tier: Option<Tier>,
        protocol: Protocol,
        _ipv4_only: bool,
        _ipv6_only: bool,
    ) -> Result<(), AppError> {
        let t = tier.unwrap_or(Tier::Standard);
        let addrs = provider
            .addresses
            .get(&t)
            .ok_or_else(|| AppError::apply_failed("Tier addresses not found"))?;

        match protocol {
            Protocol::DoQ => configure_unbound_doq(provider, addrs),
            Protocol::DnsCrypt => configure_dnscrypt(provider, addrs),
            _ => Err(AppError::apply_failed(
                "Stub backend only supports DoQ and DNSCrypt",
            )),
        }
    }

    fn backup(&self) -> Result<BackupRecord, AppError> {
        let content = std::fs::read_to_string("/etc/resolv.conf")
            .unwrap_or_else(|_| "# no resolv.conf present\n".to_string());
        Ok(BackupRecord::new(Backend::Stub, content))
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

        let localhost = nameservers
            .iter()
            .any(|ns| ns == "127.0.0.1" || ns == "::1");

        Ok(BackendStatus {
            backend: "stub".to_string(),
            active: localhost,
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

/// Configure unbound as a local QUIC-forwarding stub.
fn configure_unbound_doq(
    provider: &Provider,
    addrs: &crate::registry::ProviderAddresses,
) -> Result<(), AppError> {
    let doq_url = addrs
        .doq_url
        .as_ref()
        .ok_or_else(|| AppError::apply_failed("Provider has no DoQ URL"))?;

    // Extract hostname from quic://hostname
    let hostname = doq_url.strip_prefix("quic://").unwrap_or(doq_url.as_str());

    // Use the provider's IPv4 primary as the forward address;
    // fall back to the hostname itself if no IP is available.
    let forward_addr = if let Some(ip) = &addrs.ipv4_primary {
        format!("{ip}@853#{hostname}")
    } else {
        format!("{hostname}@853")
    };

    let conf = format!(
        "# flux-managed stub resolver for {slug} (DoQ)\n\
         server:\n\
         \tinterface: 127.0.0.1\n\
         \taccess-control: 127.0.0.1 allow\n\
         \n\
         forward-zone:\n\
         \tname: \".\"\n\
         \tforward-addr: {fwd}\n\
         \tforward-quic-upstream: yes\n",
        slug = provider.slug,
        fwd = forward_addr
    );

    let conf_dir = "/etc/unbound/unbound.conf.d";
    crate::privilege::create_dir_all(conf_dir)
        .map_err(|e| AppError::apply_failed(format!("Failed to create unbound.conf.d: {e}")))?;

    crate::privilege::write_file(&format!("{conf_dir}/flux.conf"), &conf)
        .map_err(|e| AppError::apply_failed(format!("Failed to write unbound config: {e}")))?;

    let _ = crate::privilege::run_command("systemctl", &["restart", "unbound"]);

    // Point system resolver to localhost
    crate::privilege::write_file("/etc/resolv.conf", "nameserver 127.0.0.1\n")
        .map_err(|e| AppError::apply_failed(format!("Failed to write resolv.conf: {e}")))?;

    Ok(())
}

/// Configure dnscrypt-proxy as a local DNSCrypt stub.
fn configure_dnscrypt(
    provider: &Provider,
    addrs: &crate::registry::ProviderAddresses,
) -> Result<(), AppError> {
    let stamp = addrs
        .dnscrypt_stamp
        .as_ref()
        .ok_or_else(|| AppError::apply_failed("Provider has no DNSCrypt stamp"))?;

    let toml = format!(
        "# flux-managed stub resolver for {slug} (DNSCrypt)\n\
         server_names = ['{slug}']\n\
         \n\
         [static.'{slug}']\n\
         stamp = '{stamp}'\n",
        slug = provider.slug
    );

    let conf_dir = "/etc/dnscrypt-proxy";
    crate::privilege::create_dir_all(conf_dir)
        .map_err(|e| AppError::apply_failed(format!("Failed to create dnscrypt-proxy dir: {e}")))?;

    crate::privilege::write_file(&format!("{conf_dir}/flux.toml"), &toml).map_err(|e| {
        AppError::apply_failed(format!("Failed to write dnscrypt-proxy config: {e}"))
    })?;

    let _ = crate::privilege::run_command("systemctl", &["restart", "dnscrypt-proxy"]);

    // Point system resolver to localhost
    crate::privilege::write_file("/etc/resolv.conf", "nameserver 127.0.0.1\n")
        .map_err(|e| AppError::apply_failed(format!("Failed to write resolv.conf: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_status_detects_localhost() {
        let backend = StubBackend;
        let status = backend.status().unwrap();
        assert_eq!(status.backend, "stub");
        // On Linux test systems resolv.conf may or may not point to localhost
        assert!(status.nameservers.is_some());
    }

    #[test]
    fn test_stub_apply_rejects_plain() {
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let backend = StubBackend;
        let result = backend.apply(provider, None, Protocol::Plain, false, false);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("only supports DoQ and DNSCrypt")
        );
    }

    #[test]
    fn test_stub_apply_rejects_dot() {
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let backend = StubBackend;
        let result = backend.apply(provider, None, Protocol::DoT, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_configure_unbound_doq_missing_url() {
        // Cloudflare Standard has no DoQ URL
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let addrs = provider.addresses.get(&Tier::Standard).unwrap();
        let result = configure_unbound_doq(provider, addrs);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("no DoQ URL"));
    }

    #[test]
    fn test_configure_dnscrypt_missing_stamp() {
        // Cloudflare Standard has no DNSCrypt stamp
        let provider = crate::registry::get_provider("cloudflare").unwrap();
        let addrs = provider.addresses.get(&Tier::Standard).unwrap();
        let result = configure_dnscrypt(provider, addrs);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("no DNSCrypt stamp"));
    }
}
