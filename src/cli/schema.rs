// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use crate::output::mode::OutputMode;
use serde_json::json;

/// Emit JSON Schema Draft 2020-12 for the CLI surface.
pub fn emit_schema(_mode: &OutputMode) -> Result<serde_json::Value, AppError> {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://Flux.Steelbore.com/schema/flux-v0.1.0.json",
        "title": "Flux CLI Schema",
        "description": "JSON Schema for the dns CLI surface",
        "version": "0.1.0",
        "subcommands": {
            "apply": {
                "description": "Apply DNS configuration",
                "args": {
                    "provider": { "type": "string", "enum": ["google", "cloudflare", "adguard", "quad9", "opendns"] },
                    "tier": { "type": "string", "enum": ["standard", "malware", "family", "unfiltered", "ecs", "unsecured", "secured"] },
                    "protocol": { "type": "string", "enum": ["plain", "dot", "doh", "doq", "dnscrypt", "warp"] },
                    "ipv4_only": { "type": "boolean" },
                    "ipv6_only": { "type": "boolean" },
                    "ntp": { "type": "boolean" },
                    "vpn": { "type": "string", "enum": ["warp", "adguard"] },
                    "no_backup": { "type": "boolean" },
                    "no_verify": { "type": "boolean" }
                }
            },
            "status": { "description": "Show current DNS/NTP/VPN state" },
            "list": {
                "description": "List providers, tiers, protocols, or VPN clients",
                "args": {
                    "providers": { "type": "boolean" },
                    "tiers": { "type": "boolean" },
                    "protocols": { "type": "boolean" },
                    "vpn": { "type": "boolean" }
                }
            },
            "restore": { "description": "Revert to most recent backup" },
            "verify": { "description": "Test current DNS resolution" },
            "detect": { "description": "Display detected backend info" },
            "backup": { "description": "Create a backup of current state" },
            "ntp": { "description": "Configure NTP independently" },
            "vpn": { "description": "Manage VPN" },
            "schema": { "description": "Emit JSON Schema" },
            "describe": { "description": "Emit human + machine manifest" },
            "mcp": { "description": "Launch MCP server" },
            "update_registry": { "description": "Fetch latest provider registry (v0.2+)" }
        },
        "exit_codes": {
            "0": "Success",
            "1": "General failure",
            "2": "Usage error",
            "3": "Resource not found (DETECTION_FAILED)",
            "4": "Permission denied (ELEVATION_FAILED)",
            "5": "Conflict",
            "6": "Apply failure",
            "7": "Verification failure",
            "8": "VPN error",
            "9": "Registry fetch error (v0.2+)"
        }
    });

    Ok(schema)
}
