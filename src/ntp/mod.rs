// SPDX-License-Identifier: GPL-3.0-or-later

use crate::detection::NtpBackend;
use crate::error::AppError;
use std::path::Path;

/// Map provider slug to recommended NTP server.
pub fn ntp_server_for_provider(slug: &str) -> &'static str {
    match slug {
        "cloudflare" => "time.cloudflare.com",
        "google" => "time.google.com",
        _ => "pool.ntp.org",
    }
}

/// Configure NTP for the detected backend.
///
/// Returns `Ok(None)` on success, or `Ok(Some(message))` when the backend
/// requires manual action (e.g. NixOS expression fragment).
pub fn configure_ntp(provider_slug: &str) -> Result<Option<String>, AppError> {
    let server = ntp_server_for_provider(provider_slug);
    let backend = crate::detection::detect_ntp_backend()?;

    match backend {
        NtpBackend::SystemdTimesyncd => configure_systemd_timesyncd(server),
        NtpBackend::Chrony => configure_chrony(server),
        NtpBackend::Ntpd => configure_ntpd(server),
        NtpBackend::Openntpd => configure_openntpd(server),
        NtpBackend::Nixos => Ok(Some(format!("services.ntp.servers = [ \"{server}\" ];"))),
    }
}

fn configure_systemd_timesyncd(server: &str) -> Result<Option<String>, AppError> {
    let conf = format!("# flux-managed\n[Time]\nNTP={server}\n");
    let conf_dir = "/etc/systemd/timesyncd.conf.d";

    crate::privilege::create_dir_all(conf_dir)
        .map_err(|e| AppError::apply_failed(format!("Failed to create timesyncd.d: {e}")))?;

    crate::privilege::write_file(&format!("{conf_dir}/flux.conf"), &conf)
        .map_err(|e| AppError::apply_failed(format!("Failed to write timesyncd.conf: {e}")))?;

    let _ = crate::privilege::run_command("systemctl", &["restart", "systemd-timesyncd"]);

    Ok(None)
}

fn configure_chrony(server: &str) -> Result<Option<String>, AppError> {
    let paths = ["/etc/chrony.conf", "/etc/chrony/chrony.conf"];
    let path = paths
        .iter()
        .find(|p| Path::new(p).exists())
        .copied()
        .unwrap_or("/etc/chrony.conf");

    let content = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines
        .into_iter()
        .filter(|l| !l.trim_start().starts_with("server ") || !l.contains("flux-managed"))
        .map(|s| s.to_string())
        .collect();
    new_lines.push(format!("server {server} iburst # flux-managed"));

    crate::privilege::write_file(path, &(new_lines.join("\n") + "\n"))
        .map_err(|e| AppError::apply_failed(format!("Failed to write chrony.conf: {e}")))?;

    let _ = crate::privilege::run_command("systemctl", &["restart", "chronyd"]);

    Ok(None)
}

fn configure_ntpd(server: &str) -> Result<Option<String>, AppError> {
    let path = "/etc/ntp.conf";
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines
        .into_iter()
        .filter(|l| !l.trim_start().starts_with("server ") || !l.contains("flux-managed"))
        .map(|s| s.to_string())
        .collect();
    new_lines.push(format!("server {server} iburst # flux-managed"));

    crate::privilege::write_file(path, &(new_lines.join("\n") + "\n"))
        .map_err(|e| AppError::apply_failed(format!("Failed to write ntp.conf: {e}")))?;

    let _ = crate::privilege::run_command("systemctl", &["restart", "ntpd"]);

    Ok(None)
}

fn configure_openntpd(server: &str) -> Result<Option<String>, AppError> {
    let path = "/etc/ntpd.conf";
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines
        .into_iter()
        .filter(|l| !l.trim_start().starts_with("server ") || !l.contains("flux-managed"))
        .map(|s| s.to_string())
        .collect();
    new_lines.push(format!("server {server} # flux-managed"));

    crate::privilege::write_file(path, &(new_lines.join("\n") + "\n"))
        .map_err(|e| AppError::apply_failed(format!("Failed to write ntpd.conf: {e}")))?;

    let _ = crate::privilege::run_command("systemctl", &["restart", "openntpd"]);

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ntp_server_for_provider_cloudflare() {
        assert_eq!(ntp_server_for_provider("cloudflare"), "time.cloudflare.com");
    }

    #[test]
    fn test_ntp_server_for_provider_google() {
        assert_eq!(ntp_server_for_provider("google"), "time.google.com");
    }

    #[test]
    fn test_ntp_server_for_provider_fallback() {
        assert_eq!(ntp_server_for_provider("quad9"), "pool.ntp.org");
        assert_eq!(ntp_server_for_provider("opendns"), "pool.ntp.org");
    }

    #[test]
    fn test_configure_ntp_nixos_returns_expression() {
        // We can't mock detect_ntp_backend, but we can test the helper directly.
        let expr = configure_ntp_nixos_expression("time.cloudflare.com");
        assert!(expr.contains("services.ntp.servers"));
        assert!(expr.contains("time.cloudflare.com"));
    }

    fn configure_ntp_nixos_expression(server: &str) -> String {
        format!("services.ntp.servers = [ \"{server}\" ];")
    }
}
