// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backends::DnsBackend;
use crate::detection::Backend;
use crate::error::AppError;
use crate::output::mode::OutputMode;
use crate::registry::{Protocol, Tier};

/// Orchestrate the full apply flow: backup → detect → configure → verify.
#[allow(clippy::too_many_arguments)]
pub fn run_apply(
    provider_slug: &str,
    tier: Option<Tier>,
    protocol: Protocol,
    ipv4_only: bool,
    ipv6_only: bool,
    no_backup: bool,
    no_verify: bool,
    ntp: bool,
    vpn: Option<&str>,
    mode: &OutputMode,
) -> Result<serde_json::Value, AppError> {
    let provider = crate::registry::get_provider(provider_slug)
        .ok_or_else(|| AppError::usage_error(format!("Unknown provider: {provider_slug}")))?;

    crate::registry::validate_combination(provider, tier, protocol)?;

    let backend = crate::detection::detect_backend()?;

    if !no_backup && !mode.dry_run {
        let mgr = crate::backups::BackupManager::new()?;
        let record = match backend {
            Backend::ResolvConf => crate::backends::resolvconf::ResolvConfBackend.backup()?,
            Backend::SystemdResolved => {
                crate::backends::systemd::SystemdResolvedBackend.backup()?
            }
            Backend::NetworkManager => {
                crate::backends::networkmanager::NetworkManagerBackend.backup()?
            }
            Backend::Nixos => crate::backends::nixos::NixosBackend.backup()?,
            Backend::Resolvectl => crate::backends::resolvectl::ResolvectlBackend.backup()?,
            Backend::FreeBsd => crate::backends::bsd::FreeBsdBackend.backup()?,
            Backend::OpenBsd => crate::backends::bsd::OpenBsdBackend.backup()?,
            Backend::NetBsd => crate::backends::bsd::NetBsdBackend.backup()?,
            Backend::Stub => crate::backends::stub::StubBackend.backup()?,
        };
        let _ = mgr.save(&record, Some(provider_slug));
    }

    if mode.dry_run {
        return Ok(serde_json::json!({
            "action": "apply",
            "provider": provider_slug,
            "tier": tier.map(|t| t.to_string()),
            "protocol": protocol.to_string(),
            "backend": backend.to_string(),
            "dry_run": true
        }));
    }

    // Use Stub backend as fallback for protocols the detected backend cannot handle.
    let effective_backend = if crate::registry::backend_supports_protocol(backend, protocol) {
        backend
    } else {
        Backend::Stub
    };

    // Apply via backend
    match effective_backend {
        Backend::ResolvConf => {
            let b = crate::backends::resolvconf::ResolvConfBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::SystemdResolved => {
            let b = crate::backends::systemd::SystemdResolvedBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::NetworkManager => {
            let b = crate::backends::networkmanager::NetworkManagerBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::Nixos => {
            let b = crate::backends::nixos::NixosBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::Resolvectl => {
            let b = crate::backends::resolvectl::ResolvectlBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::FreeBsd => {
            let b = crate::backends::bsd::FreeBsdBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::OpenBsd => {
            let b = crate::backends::bsd::OpenBsdBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::NetBsd => {
            let b = crate::backends::bsd::NetBsdBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
        Backend::Stub => {
            let b = crate::backends::stub::StubBackend;
            b.apply(provider, tier, protocol, ipv4_only, ipv6_only)?;
        }
    }

    // Verify
    let verify = if !no_verify {
        let result = match effective_backend {
            Backend::ResolvConf => {
                let b = crate::backends::resolvconf::ResolvConfBackend;
                b.verify(5)?
            }
            Backend::SystemdResolved => {
                let b = crate::backends::systemd::SystemdResolvedBackend;
                b.verify(5)?
            }
            Backend::NetworkManager => {
                let b = crate::backends::networkmanager::NetworkManagerBackend;
                b.verify(5)?
            }
            Backend::Nixos => {
                let b = crate::backends::nixos::NixosBackend;
                b.verify(5)?
            }
            Backend::Resolvectl => {
                let b = crate::backends::resolvectl::ResolvectlBackend;
                b.verify(5)?
            }
            Backend::FreeBsd => {
                let b = crate::backends::bsd::FreeBsdBackend;
                b.verify(5)?
            }
            Backend::OpenBsd => {
                let b = crate::backends::bsd::OpenBsdBackend;
                b.verify(5)?
            }
            Backend::NetBsd => {
                let b = crate::backends::bsd::NetBsdBackend;
                b.verify(5)?
            }
            Backend::Stub => {
                let b = crate::backends::stub::StubBackend;
                b.verify(5)?
            }
        };
        if !result.success {
            return Err(AppError::verification_failed(
                result
                    .error
                    .unwrap_or_else(|| "DNS verification failed".to_string()),
            ));
        }
        Some(result)
    } else {
        None
    };

    // Optional NTP configuration
    let ntp_result = if ntp {
        match crate::ntp::configure_ntp(provider_slug) {
            Ok(Some(expr)) => Some(serde_json::json!({
                "configured": true,
                "manual": true,
                "expression": expr,
            })),
            Ok(None) => Some(serde_json::json!({
                "configured": true,
                "manual": false,
            })),
            Err(e) => Some(serde_json::json!({
                "configured": false,
                "error": e.message,
            })),
        }
    } else {
        None
    };

    // Optional VPN connection
    let vpn_status = if let Some(vpn_slug) = vpn {
        let provider = crate::vpn::get_vpn_provider(vpn_slug)
            .ok_or_else(|| AppError::usage_error(format!("Unknown VPN provider: {vpn_slug}")))?;
        let args = crate::vpn::VpnConnectArgs {
            license: None,
            location: None,
            protocol: None,
        };
        provider.connect(&args)?;
        let status = provider.status()?;
        Some(serde_json::json!({
            "provider": vpn_slug,
            "connected": status.connected,
            "protocol": status.protocol,
        }))
    } else {
        None
    };

    Ok(serde_json::json!({
        "provider": provider_slug,
        "tier": tier.map(|t| t.to_string()),
        "protocol": protocol.to_string(),
        "backend": backend.to_string(),
        "verify": verify,
        "ntp": ntp_result,
        "vpn": vpn_status,
    }))
}
