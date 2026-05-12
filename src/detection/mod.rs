// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Detected DNS backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Nixos,
    FreeBsd,
    OpenBsd,
    NetBsd,
    SystemdResolved,
    NetworkManager,
    Resolvectl,
    ResolvConf,
    Stub,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Backend::Nixos => "nixos",
            Backend::FreeBsd => "freebsd",
            Backend::OpenBsd => "openbsd",
            Backend::NetBsd => "netbsd",
            Backend::SystemdResolved => "systemd-resolved",
            Backend::NetworkManager => "networkmanager",
            Backend::Resolvectl => "resolvectl",
            Backend::ResolvConf => "resolv.conf",
            Backend::Stub => "stub",
        };
        write!(f, "{s}")
    }
}

/// Detected NTP backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NtpBackend {
    SystemdTimesyncd,
    Chrony,
    Ntpd,
    Openntpd,
    Nixos,
}

/// Detect the active DNS backend.
///
/// Priority order per PRD §7.1:
/// 1. NixOS → BSD → systemd-resolved → NetworkManager → resolvectl → /etc/resolv.conf
pub fn detect_backend() -> Result<Backend, AppError> {
    // 1. NixOS
    if std::path::Path::new("/etc/NIXOS").exists() {
        return Ok(Backend::Nixos);
    }

    // 2. BSD
    if let Ok(uts) = nix::sys::utsname::uname() {
        let sysname = uts.sysname().to_str().unwrap_or("");
        match sysname {
            "FreeBSD" => return Ok(Backend::FreeBsd),
            "OpenBSD" => return Ok(Backend::OpenBsd),
            "NetBSD" => return Ok(Backend::NetBsd),
            _ => {}
        }
    }

    // 3. systemd-resolved
    if cmd_success("systemctl", &["is-active", "systemd-resolved"]) {
        return Ok(Backend::SystemdResolved);
    }

    // 4. NetworkManager
    if cmd_success("systemctl", &["is-active", "NetworkManager"]) && which("nmcli").is_some() {
        return Ok(Backend::NetworkManager);
    }

    // 5. resolvectl
    if cmd_success("resolvectl", &["status"]) {
        return Ok(Backend::Resolvectl);
    }

    // 6. /etc/resolv.conf
    let resolv = std::path::Path::new("/etc/resolv.conf");
    if resolv.exists() {
        // Check it's not symlinked to systemd stub
        if let Ok(meta) = std::fs::symlink_metadata(resolv) {
            if meta.file_type().is_symlink() {
                if let Ok(target) = std::fs::read_link(resolv) {
                    let target_str = target.to_string_lossy();
                    if target_str.contains("systemd") || target_str.contains("127.0.0.53") {
                        // Fall through — systemd-resolved already checked above
                    } else {
                        return Ok(Backend::ResolvConf);
                    }
                }
            } else {
                return Ok(Backend::ResolvConf);
            }
        }
    }

    Err(AppError::detection_failed(
        "No supported DNS subsystem detected on this system",
    ))
}

/// Detect the active NTP backend.
pub fn detect_ntp_backend() -> Result<NtpBackend, AppError> {
    if std::path::Path::new("/etc/NIXOS").exists() {
        return Ok(NtpBackend::Nixos);
    }

    if cmd_success("systemctl", &["is-active", "systemd-timesyncd"]) {
        return Ok(NtpBackend::SystemdTimesyncd);
    }

    if which("chronyc").is_some() || std::path::Path::new("/etc/chrony.conf").exists() {
        return Ok(NtpBackend::Chrony);
    }

    if which("ntpd").is_some() || std::path::Path::new("/etc/ntp.conf").exists() {
        return Ok(NtpBackend::Ntpd);
    }

    if which("openntpd").is_some() || std::path::Path::new("/etc/ntpd.conf").exists() {
        return Ok(NtpBackend::Openntpd);
    }

    Err(AppError::detection_failed(
        "No supported NTP subsystem detected on this system",
    ))
}

fn cmd_success(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(cmd))
            .find(|p| p.is_file())
    })
}

/// Package manager detected on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManager {
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Nix,
    Pkg,    // FreeBSD
    PkgAdd, // OpenBSD
    Pkgsrc, // NetBSD
}

/// Detect the system's package manager.
pub fn detect_package_manager() -> Option<PackageManager> {
    if which("pacman").is_some() {
        Some(PackageManager::Pacman)
    } else if which("apt-get").is_some() || which("apt").is_some() {
        Some(PackageManager::Apt)
    } else if which("dnf").is_some() {
        Some(PackageManager::Dnf)
    } else if which("zypper").is_some() {
        Some(PackageManager::Zypper)
    } else if which("nix").is_some() {
        Some(PackageManager::Nix)
    } else if which("pkg").is_some() {
        Some(PackageManager::Pkg)
    } else if which("pkg_add").is_some() {
        Some(PackageManager::PkgAdd)
    } else {
        None
    }
}
