// SPDX-License-Identifier: GPL-3.0-or-later

use super::{VpnConnectArgs, VpnProvider, VpnStatus, which};
use crate::error::AppError;

pub struct WarpProvider;

impl VpnProvider for WarpProvider {
    fn connect(&self, args: &VpnConnectArgs) -> Result<(), AppError> {
        args.validate()?;

        if !self.is_available() {
            return Err(AppError::vpn_error(
                "warp-cli not found. Install: https://developers.cloudflare.com/warp-client/",
            ));
        }

        // Register if needed
        let _ = std::process::Command::new("warp-cli")
            .args(["register"])
            .status();

        if let Some(license) = &args.license {
            let _ = std::process::Command::new("warp-cli")
                .args(["set-license", license])
                .status();
        }

        let output = std::process::Command::new("warp-cli")
            .args(["connect"])
            .output()
            .map_err(|e| AppError::vpn_error(format!("Failed to start warp-cli: {e}")))?;

        if !output.status.success() {
            return Err(AppError::vpn_error(format!(
                "warp-cli connect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn disconnect(&self) -> Result<(), AppError> {
        let output = std::process::Command::new("warp-cli")
            .args(["disconnect"])
            .output()
            .map_err(|e| AppError::vpn_error(format!("Failed to run warp-cli: {e}")))?;

        if !output.status.success() {
            return Err(AppError::vpn_error(format!(
                "warp-cli disconnect failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    fn status(&self) -> Result<VpnStatus, AppError> {
        let output = std::process::Command::new("warp-cli")
            .args(["status"])
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let connected = text.contains("Connected");
                Ok(VpnStatus {
                    provider: "warp".to_string(),
                    connected,
                    endpoint: None,
                    protocol: Some("wireguard".to_string()),
                })
            }
            Err(e) => Err(AppError::vpn_error(format!("warp-cli status failed: {e}"))),
        }
    }

    fn is_available(&self) -> bool {
        which("warp-cli").is_some()
    }
}
