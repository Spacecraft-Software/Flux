// SPDX-License-Identifier: GPL-3.0-or-later

use super::{VpnConnectArgs, VpnProvider, VpnStatus, which};
use crate::error::AppError;

pub struct AdGuardVpnProvider;

impl VpnProvider for AdGuardVpnProvider {
    fn connect(&self, args: &VpnConnectArgs) -> Result<(), AppError> {
        args.validate()?;

        if !self.is_available() {
            return Err(AppError::vpn_error(
                "adguardvpn-cli not found. See https://adguard-vpn.com/en/welcome.html",
            ));
        }

        // Login if needed (first-run)
        let _ = std::process::Command::new("adguardvpn-cli")
            .args(["login"])
            .status();

        let mut cmd_args = vec!["connect"];
        if let Some(loc) = &args.location {
            cmd_args.push("--location");
            cmd_args.push(loc);
        }

        let output = std::process::Command::new("adguardvpn-cli")
            .args(&cmd_args)
            .output()
            .map_err(|e| AppError::vpn_error(format!("Failed to start adguardvpn-cli: {e}")))?;

        if !output.status.success() {
            return Err(AppError::vpn_error(format!(
                "adguardvpn-cli connect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn disconnect(&self) -> Result<(), AppError> {
        let output = std::process::Command::new("adguardvpn-cli")
            .args(["disconnect"])
            .output()
            .map_err(|e| AppError::vpn_error(format!("Failed to run adguardvpn-cli: {e}")))?;

        if !output.status.success() {
            return Err(AppError::vpn_error(format!(
                "adguardvpn-cli disconnect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn status(&self) -> Result<VpnStatus, AppError> {
        let output = std::process::Command::new("adguardvpn-cli")
            .args(["status"])
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let connected = text.contains("Connected");
                Ok(VpnStatus {
                    provider: "adguard".to_string(),
                    connected,
                    endpoint: None,
                    protocol: Some("quic".to_string()),
                })
            }
            Err(e) => Err(AppError::vpn_error(format!(
                "adguardvpn-cli status failed: {e}"
            ))),
        }
    }

    fn is_available(&self) -> bool {
        which("adguardvpn-cli").is_some()
    }
}
