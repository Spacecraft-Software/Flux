// SPDX-License-Identifier: GPL-3.0-or-later

use crate::output::mode::{ColorArg, FormatArg};
use clap::{Parser, Subcommand};

pub mod schema;

/// dns — DNS selector and network configurator
#[derive(Parser, Debug)]
#[command(name = "dns")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "DNS selector and network configurator for Linux and BSD")]
#[command(long_about = concat!(
    "Flux — DNS Selector & Network Configurator\n",
    "\n",
    "Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>\n",
    "https://Flux.SpacecraftSoftware.org/"
))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output format
    #[arg(long, value_enum, global = true)]
    pub format: Option<FormatArg>,

    /// Alias for --format json
    #[arg(long, global = true)]
    pub json: bool,

    /// Comma-separated field selection
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// Emit action plan without side effects
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Verbose diagnostic output to stderr
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-error stderr
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable ANSI color
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Color mode
    #[arg(long, value_enum, global = true)]
    pub color: Option<ColorArg>,

    /// Disable relative-time rendering in human mode
    #[arg(long, global = true)]
    pub absolute_time: bool,

    /// NUL-delimited output
    #[arg(short = '0', long, global = true)]
    pub print0: bool,

    /// Skip confirmation in non-TTY mode
    #[arg(long, global = true)]
    pub yes: bool,

    /// Force destructive operations
    #[arg(long, global = true)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Apply DNS configuration
    Apply(ApplyArgs),

    /// Show current DNS/NTP/VPN state
    Status,

    /// List providers, tiers, protocols, or VPN clients
    List(ListArgs),

    /// Revert to most recent backup
    Restore,

    /// Test current DNS resolution
    Verify,

    /// Display detected backend info
    Detect,

    /// Create a backup of current state
    Backup,

    /// Configure NTP independently
    Ntp(NtpArgs),

    /// Manage VPN (connect / disconnect / status)
    Vpn(VpnArgs),

    /// Emit JSON Schema for the CLI surface
    Schema,

    /// Emit human + machine manifest of the CLI
    Describe,

    /// Launch MCP server
    Mcp,

    /// Fetch latest provider registry (v0.2+)
    UpdateRegistry,
}

#[derive(Parser, Debug)]
pub struct ApplyArgs {
    /// Provider slug
    #[arg(short, long)]
    pub provider: Option<String>,

    /// Filtering tier
    #[arg(short, long)]
    pub tier: Option<String>,

    /// Transport protocol
    #[arg(short = 'P', long)]
    pub protocol: Option<String>,

    /// IPv4 only
    #[arg(short = '4', long)]
    pub ipv4_only: bool,

    /// IPv6 only
    #[arg(short = '6', long)]
    pub ipv6_only: bool,

    /// Also configure NTP
    #[arg(long)]
    pub ntp: bool,

    /// Also set up VPN client
    #[arg(long)]
    pub vpn: Option<String>,

    /// Skip backup creation
    #[arg(long)]
    pub no_backup: bool,

    /// Skip post-apply verification
    #[arg(long)]
    pub no_verify: bool,

    /// Positional arguments: [provider] [tier] [protocol]
    #[arg(value_name = "ARGS")]
    pub positional: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ListArgs {
    /// List providers
    #[arg(long)]
    pub providers: bool,

    /// List tiers for a provider
    #[arg(long)]
    pub tiers: bool,

    /// List protocols for a provider
    #[arg(long)]
    pub protocols: bool,

    /// List VPN clients
    #[arg(long)]
    pub vpn: bool,

    /// Provider slug (for --tiers or --protocols)
    #[arg(short, long)]
    pub provider: Option<String>,

    /// Tier (for --protocols)
    #[arg(short, long)]
    pub tier: Option<String>,
}

#[derive(Parser, Debug)]
pub struct NtpArgs {
    /// Provider slug for NTP server mapping
    #[arg(short, long)]
    pub provider: Option<String>,
}

#[derive(Parser, Debug)]
pub struct VpnArgs {
    #[command(subcommand)]
    pub action: VpnAction,
}

#[derive(Subcommand, Debug)]
pub enum VpnAction {
    /// Connect to VPN
    Connect {
        /// Provider: warp or adguard
        #[arg(short, long)]
        provider: String,
        /// WARP+ license key
        #[arg(long)]
        license: Option<String>,
        /// Location (AdGuard only)
        #[arg(long)]
        location: Option<String>,
    },
    /// Disconnect from VPN
    Disconnect {
        /// Provider: warp or adguard
        #[arg(short, long)]
        provider: String,
    },
    /// Show VPN status
    Status,
}
