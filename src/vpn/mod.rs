// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Shared VPN provider trait.
pub trait VpnProvider {
    fn connect(&self, args: &VpnConnectArgs) -> Result<(), AppError>;
    fn disconnect(&self) -> Result<(), AppError>;
    fn status(&self) -> Result<VpnStatus, AppError>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct VpnConnectArgs {
    pub license: Option<String>,
    pub location: Option<String>,
    #[allow(dead_code)]
    pub protocol: Option<String>,
}

impl VpnConnectArgs {
    /// Validate all string fields for control characters and ANSI escapes.
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(ref s) = self.license {
            crate::validate::reject_control_chars(s)?;
        }
        if let Some(ref s) = self.location {
            crate::validate::reject_control_chars(s)?;
        }
        if let Some(ref s) = self.protocol {
            crate::validate::reject_control_chars(s)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnStatus {
    pub provider: String,
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
}

pub mod adguard;
pub mod warp;

/// Get VPN provider by slug.
pub fn get_vpn_provider(slug: &str) -> Option<Box<dyn VpnProvider>> {
    match slug {
        "warp" | "cloudflare" => Some(Box::new(warp::WarpProvider)),
        "adguard" => Some(Box::new(adguard::AdGuardVpnProvider)),
        _ => None,
    }
}

/// Locate a command on PATH.
pub fn which(cmd: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(cmd))
            .find(|p| p.is_file())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpn_connect_args_validate_ok() {
        let args = VpnConnectArgs {
            license: Some("ABC-123".to_string()),
            location: Some("us-east".to_string()),
            protocol: None,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_vpn_connect_args_validate_rejects_control_char() {
        let args = VpnConnectArgs {
            license: Some("ABC\x07123".to_string()),
            location: None,
            protocol: None,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_vpn_connect_args_validate_rejects_ansi() {
        let args = VpnConnectArgs {
            license: Some("\x1b[31mred".to_string()),
            location: None,
            protocol: None,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_get_vpn_provider_warp() {
        let p = get_vpn_provider("warp");
        assert!(p.is_some());
        assert!(p.unwrap().status().is_err()); // warp-cli not installed in test env
    }

    #[test]
    fn test_get_vpn_provider_adguard() {
        let p = get_vpn_provider("adguard");
        assert!(p.is_some());
    }

    #[test]
    fn test_get_vpn_provider_unknown() {
        assert!(get_vpn_provider("nordvpn").is_none());
    }

    #[test]
    fn test_which_finds_common_commands() {
        // `true` exists on every Unix system.
        assert!(which("true").is_some());
        // Something extremely unlikely to exist.
        assert!(which("flux-vpn-not-real-xyz").is_none());
    }
}
