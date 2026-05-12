// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::result_large_err)]

use clap::Parser;

mod agent;
mod backends;
mod backups;
mod cli;
mod detection;
mod error;
mod ntp;
mod orchestrator;
mod output;
mod privilege;
mod registry;
#[cfg(feature = "tui")]
mod tui;

#[cfg(not(feature = "tui"))]
mod tui {
    pub fn run_tui() -> Result<(), crate::error::AppError> {
        Err(crate::error::AppError::general(
            "TUI feature not compiled in",
        ))
    }
}
mod validate;
mod vpn;

use backends::DnsBackend;
use cli::{Cli, Commands};
use error::{AppError, ExitCode};
use output::mode::OutputMode;
use vpn::VpnProvider;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mode = OutputMode::from_args(
        cli.format,
        cli.json,
        cli.no_color,
        cli.color,
        cli.dry_run,
        cli.verbose,
        cli.quiet,
        cli.yes || cli.force,
        cli.fields,
        cli.absolute_time,
        cli.print0,
    );

    let result = match cli.command {
        None => {
            if mode.interactive {
                tui::run_tui().map(|_| serde_json::json!({"tui": "exited"}))
            } else {
                run_cli(None, &mode).await
            }
        }
        Some(cmd) => run_cli(Some(cmd), &mode).await,
    };

    match result {
        Ok(data) => {
            output::emit_data(&data, &mode);
            std::process::exit(ExitCode::Success.as_i32());
        }
        Err(err) => {
            output::emit_error(&err, &mode);
            std::process::exit(err.exit_code);
        }
    }
}

async fn run_cli(cmd: Option<Commands>, mode: &OutputMode) -> Result<serde_json::Value, AppError> {
    match cmd {
        None => {
            // No subcommand: in non-interactive mode, print help or status
            let status = serde_json::json!({
                "message": "Use 'dns --help' for usage or run interactively for TUI"
            });
            Ok(status)
        }
        Some(Commands::Apply(args)) => {
            let (provider, tier, protocol) = parse_apply_args(&args)?;
            orchestrator::run_apply(
                &provider,
                tier,
                protocol,
                args.ipv4_only,
                args.ipv6_only,
                args.no_backup,
                args.no_verify,
                args.ntp,
                args.vpn.as_deref(),
                mode,
            )
        }
        Some(Commands::Status) => {
            let backend = detection::detect_backend()?;
            let mut vpn_statuses = Vec::new();
            let warp = vpn::warp::WarpProvider;
            if warp.is_available() {
                if let Ok(st) = warp.status() {
                    vpn_statuses.push(serde_json::json!({
                        "provider": st.provider,
                        "connected": st.connected,
                        "protocol": st.protocol,
                    }));
                }
            }
            let adguard = vpn::adguard::AdGuardVpnProvider;
            if adguard.is_available() {
                if let Ok(st) = adguard.status() {
                    vpn_statuses.push(serde_json::json!({
                        "provider": st.provider,
                        "connected": st.connected,
                        "protocol": st.protocol,
                    }));
                }
            }
            let status = serde_json::json!({
                "backend": backend.to_string(),
                "dns": get_backend_status(backend)?,
                "vpn": vpn_statuses,
            });
            Ok(status)
        }
        Some(Commands::List(args)) => {
            let data = if args.providers {
                let providers: Vec<_> = registry::list_providers()
                    .into_iter()
                    .map(|p| {
                        serde_json::json!({
                            "slug": p.slug,
                            "name": p.name,
                            "tiers": p.tiers.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                            "protocols": p.protocols.iter().map(|pr| pr.to_string()).collect::<Vec<_>>(),
                            "ntp_server": p.ntp_server,
                        })
                    })
                    .collect();
                serde_json::json!({ "providers": providers })
            } else if args.tiers {
                let slug = args.provider.as_deref().unwrap_or("cloudflare");
                let provider = registry::get_provider(slug)
                    .ok_or_else(|| AppError::usage_error(format!("Unknown provider: {slug}")))?;
                let tiers: Vec<_> = provider.tiers.iter().map(|t| t.to_string()).collect();
                serde_json::json!({ "provider": slug, "tiers": tiers })
            } else if args.protocols {
                let slug = args.provider.as_deref().unwrap_or("cloudflare");
                let provider = registry::get_provider(slug)
                    .ok_or_else(|| AppError::usage_error(format!("Unknown provider: {slug}")))?;
                let tier = args.tier.as_deref().map(|t| t.parse()).transpose()?;
                let backend = detection::detect_backend()?;
                let protocols: Vec<_> = registry::valid_protocols(provider, tier, backend)
                    .iter()
                    .map(|p| p.to_string())
                    .collect();
                serde_json::json!({ "provider": slug, "tier": tier.map(|t| t.to_string()), "protocols": protocols })
            } else if args.vpn {
                let warp = vpn::warp::WarpProvider;
                let adguard = vpn::adguard::AdGuardVpnProvider;
                serde_json::json!({
                    "vpn_clients": [
                        { "name": "warp", "available": warp.is_available() },
                        { "name": "adguard", "available": adguard.is_available() }
                    ]
                })
            } else {
                serde_json::json!({
                    "providers": registry::list_providers().len(),
                    "message": "Use --providers, --tiers, --protocols, or --vpn"
                })
            };
            Ok(data)
        }
        Some(Commands::Restore) => {
            let backend = detection::detect_backend()?;
            let mgr = backups::BackupManager::new()?;
            mgr.restore(backend)?;
            Ok(serde_json::json!({ "restored": true, "backend": backend.to_string() }))
        }
        Some(Commands::Verify) => {
            let backend = detection::detect_backend()?;
            let result = match backend {
                detection::Backend::ResolvConf => {
                    crate::backends::resolvconf::ResolvConfBackend.verify(5)?
                }
                detection::Backend::SystemdResolved => {
                    crate::backends::systemd::SystemdResolvedBackend.verify(5)?
                }
                detection::Backend::NetworkManager => {
                    crate::backends::networkmanager::NetworkManagerBackend.verify(5)?
                }
                detection::Backend::Nixos => crate::backends::nixos::NixosBackend.verify(5)?,
                detection::Backend::Resolvectl => {
                    crate::backends::resolvectl::ResolvectlBackend.verify(5)?
                }
                detection::Backend::FreeBsd => crate::backends::bsd::FreeBsdBackend.verify(5)?,
                detection::Backend::OpenBsd => crate::backends::bsd::OpenBsdBackend.verify(5)?,
                detection::Backend::NetBsd => crate::backends::bsd::NetBsdBackend.verify(5)?,
                detection::Backend::Stub => crate::backends::stub::StubBackend.verify(5)?,
            };
            Ok(serde_json::to_value(result).map_err(|e| AppError::general(e.to_string()))?)
        }
        Some(Commands::Detect) => {
            let backend = detection::detect_backend()?;
            let ntp = detection::detect_ntp_backend().ok();
            let pkg_mgr = detection::detect_package_manager();
            Ok(serde_json::json!({
                "dns_backend": backend.to_string(),
                "ntp_backend": ntp.map(|b| format!("{b:?}")),
                "package_manager": pkg_mgr.map(|p| format!("{p:?}")),
            }))
        }
        Some(Commands::Backup) => {
            let backend = detection::detect_backend()?;
            let mgr = backups::BackupManager::new()?;
            let record = match backend {
                detection::Backend::ResolvConf => {
                    crate::backends::resolvconf::ResolvConfBackend.backup()?
                }
                detection::Backend::SystemdResolved => {
                    crate::backends::systemd::SystemdResolvedBackend.backup()?
                }
                detection::Backend::NetworkManager => {
                    crate::backends::networkmanager::NetworkManagerBackend.backup()?
                }
                detection::Backend::Nixos => crate::backends::nixos::NixosBackend.backup()?,
                detection::Backend::Resolvectl => {
                    crate::backends::resolvectl::ResolvectlBackend.backup()?
                }
                detection::Backend::FreeBsd => crate::backends::bsd::FreeBsdBackend.backup()?,
                detection::Backend::OpenBsd => crate::backends::bsd::OpenBsdBackend.backup()?,
                detection::Backend::NetBsd => crate::backends::bsd::NetBsdBackend.backup()?,
                detection::Backend::Stub => crate::backends::stub::StubBackend.backup()?,
            };
            let path = mgr.save(&record, None)?;
            Ok(serde_json::json!({
                "backed_up": true,
                "backend": backend.to_string(),
                "path": path.to_string_lossy()
            }))
        }
        Some(Commands::Ntp(args)) => {
            let provider = args.provider.as_deref().unwrap_or("cloudflare");
            validate::reject_prompt_injection(provider)?;
            let result = ntp::configure_ntp(provider)?;
            Ok(serde_json::json!({
                "ntp_configured": true,
                "provider": provider,
                "manual": result.is_some(),
                "expression": result,
            }))
        }
        Some(Commands::Vpn(args)) => match args.action {
            cli::VpnAction::Connect {
                provider,
                license,
                location,
            } => {
                validate::reject_prompt_injection(&provider)?;
                if let Some(ref lic) = license {
                    validate::reject_prompt_injection(lic)?;
                }
                if let Some(ref loc) = location {
                    validate::reject_prompt_injection(loc)?;
                }
                let vpn = vpn::get_vpn_provider(&provider).ok_or_else(|| {
                    AppError::usage_error(format!("Unknown VPN provider: {provider}"))
                })?;
                let conn_args = vpn::VpnConnectArgs {
                    license,
                    location,
                    protocol: None,
                };
                vpn.connect(&conn_args)?;
                Ok(serde_json::json!({ "vpn": provider, "action": "connect" }))
            }
            cli::VpnAction::Disconnect { provider } => {
                validate::reject_prompt_injection(&provider)?;
                let vpn = vpn::get_vpn_provider(&provider).ok_or_else(|| {
                    AppError::usage_error(format!("Unknown VPN provider: {provider}"))
                })?;
                vpn.disconnect()?;
                Ok(serde_json::json!({ "vpn": provider, "action": "disconnect" }))
            }
            cli::VpnAction::Status => {
                let mut statuses = Vec::new();
                if let Some(warp) = vpn::get_vpn_provider("warp") {
                    if let Ok(s) = warp.status() {
                        statuses.push(serde_json::to_value(s).unwrap());
                    }
                }
                if let Some(ag) = vpn::get_vpn_provider("adguard") {
                    if let Ok(s) = ag.status() {
                        statuses.push(serde_json::to_value(s).unwrap());
                    }
                }
                Ok(serde_json::json!({ "vpn_statuses": statuses }))
            }
        },
        Some(Commands::Schema) => {
            let schema = cli::schema::emit_schema(mode)?;
            Ok(schema)
        }
        Some(Commands::Describe) => {
            let manifest = serde_json::json!({
                "tool": "flux",
                "version": env!("CARGO_PKG_VERSION"),
                "subcommands": [
                    { "name": "apply", "class": "apply", "tags": ["write"] },
                    { "name": "status", "class": "get", "tags": ["read"] },
                    { "name": "list", "class": "list", "tags": ["read"] },
                    { "name": "restore", "class": "apply", "tags": ["destructive"] },
                    { "name": "verify", "class": "get", "tags": ["read"] },
                    { "name": "detect", "class": "describe", "tags": ["read"] },
                    { "name": "backup", "class": "create", "tags": ["write"] },
                    { "name": "ntp", "class": "apply", "tags": ["write"] },
                    { "name": "vpn", "class": "apply", "tags": ["write"] },
                    { "name": "schema", "class": "describe", "tags": ["read"] },
                    { "name": "describe", "class": "describe", "tags": ["read"] },
                    { "name": "mcp", "class": "server", "tags": ["read"] },
                ]
            });
            Ok(manifest)
        }
        Some(Commands::Mcp) => {
            // MCP server owns stdout for protocol I/O; bypass normal emit/exit flow.
            if let Err(err) = agent::mcp::run_mcp_server().await {
                output::emit_error(&err, mode);
                std::process::exit(err.exit_code);
            }
            std::process::exit(ExitCode::Success.as_i32());
        }
        Some(Commands::UpdateRegistry) => Ok(serde_json::json!({ "message": "Coming in v0.2.0" })),
    }
}

fn parse_apply_args(
    args: &cli::ApplyArgs,
) -> Result<(String, Option<registry::Tier>, registry::Protocol), AppError> {
    if let (Some(p), Some(pr)) = (&args.provider, &args.protocol) {
        validate::reject_control_chars(p)?;
        validate::reject_prompt_injection(p)?;
        validate::reject_control_chars(pr)?;
        validate::reject_prompt_injection(pr)?;
        if let Some(t) = &args.tier {
            validate::reject_control_chars(t)?;
            validate::reject_prompt_injection(t)?;
        }
        let tier = args.tier.as_deref().map(|t| t.parse()).transpose()?;
        return Ok((p.clone(), tier, pr.parse()?));
    }

    // Positional parsing
    if args.positional.is_empty() {
        return Err(AppError::usage_error(
            "Provider required. Use -p/--provider or positional argument.",
        ));
    }

    let provider = args.positional[0].clone();
    validate::reject_control_chars(&provider)?;
    validate::reject_prompt_injection(&provider)?;

    if args.positional.len() == 1 {
        return Err(AppError::usage_error(
            "Protocol required. Use -P/--protocol or positional argument.",
        ));
    }

    // Disambiguate: second arg is tier if it matches a known tier, else protocol
    let second = args.positional[1].clone();
    validate::reject_control_chars(&second)?;
    validate::reject_prompt_injection(&second)?;
    let tier_result: Result<registry::Tier, _> = second.parse();

    if let Ok(tier) = tier_result {
        // Second arg is a tier, third must be protocol
        if args.positional.len() < 3 {
            return Err(AppError::usage_error("Protocol required after tier."));
        }
        let protocol_str = &args.positional[2];
        validate::reject_control_chars(protocol_str)?;
        validate::reject_prompt_injection(protocol_str)?;
        let protocol = protocol_str.parse()?;
        Ok((provider, Some(tier), protocol))
    } else {
        // Second arg is a protocol
        let protocol = second.parse()?;
        Ok((provider, None, protocol))
    }
}

fn get_backend_status(backend: detection::Backend) -> Result<serde_json::Value, AppError> {
    let status = match backend {
        detection::Backend::ResolvConf => {
            crate::backends::resolvconf::ResolvConfBackend.status()?
        }
        detection::Backend::SystemdResolved => {
            crate::backends::systemd::SystemdResolvedBackend.status()?
        }
        detection::Backend::NetworkManager => {
            crate::backends::networkmanager::NetworkManagerBackend.status()?
        }
        detection::Backend::Nixos => crate::backends::nixos::NixosBackend.status()?,
        detection::Backend::Resolvectl => {
            crate::backends::resolvectl::ResolvectlBackend.status()?
        }
        detection::Backend::FreeBsd => crate::backends::bsd::FreeBsdBackend.status()?,
        detection::Backend::OpenBsd => crate::backends::bsd::OpenBsdBackend.status()?,
        detection::Backend::NetBsd => crate::backends::bsd::NetBsdBackend.status()?,
        detection::Backend::Stub => crate::backends::stub::StubBackend.status()?,
    };
    serde_json::to_value(status).map_err(|e| AppError::general(e.to_string()))
}
