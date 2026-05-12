// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use std::path::PathBuf;
use std::process::Command;

/// Detect available elevation tool: sudo → pkexec → doas.
pub fn detect_elevation_tool() -> Option<&'static str> {
    if which("sudo").is_some() {
        Some("sudo")
    } else if which("pkexec").is_some() {
        Some("pkexec")
    } else if which("doas").is_some() {
        Some("doas")
    } else {
        None
    }
}

/// Check if currently running as root.
pub fn is_root() -> bool {
    nix::unistd::geteuid().is_root()
}

/// Write content to a system path, elevating if necessary.
///
/// Parent never runs as root. Content is staged to a temporary file under
/// `~/.local/share/flux/.tmp/` (mode 0600) and copied by an elevated child.
pub fn write_file(path: &str, content: &str) -> Result<(), AppError> {
    if is_root() {
        std::fs::write(path, content)
            .map_err(|e| AppError::apply_failed(format!("Failed to write {path}: {e}")))?;
        return Ok(());
    }

    let tmp_dir = flux_tmp_dir()?;
    let tmp_name = format!(
        "flux-write-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let tmp_path = tmp_dir.join(&tmp_name);

    std::fs::write(&tmp_path, content)
        .map_err(|e| AppError::apply_failed(format!("Failed to write temp file: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp_path)
            .map_err(|e| AppError::apply_failed(format!("Failed to get temp metadata: {e}")))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&tmp_path, perms)
            .map_err(|e| AppError::apply_failed(format!("Failed to chmod temp file: {e}")))?;
    }

    let result = elevate("cp", &[tmp_path.to_str().unwrap_or(""), path]);

    // Best-effort cleanup; ignore errors.
    let _ = std::fs::remove_file(&tmp_path);

    result
}

/// Create a directory and all parents, elevating if necessary.
pub fn create_dir_all(path: &str) -> Result<(), AppError> {
    if is_root() {
        std::fs::create_dir_all(path)
            .map_err(|e| AppError::apply_failed(format!("Failed to create dir {path}: {e}")))?;
        return Ok(());
    }
    elevate("mkdir", &["-p", path])
}

/// Remove a file, elevating if necessary.
pub fn remove_file(path: &str) -> Result<(), AppError> {
    if is_root() {
        let _ = std::fs::remove_file(path);
        return Ok(());
    }
    elevate("rm", &["-f", path])
}

/// Run an external command, elevating if necessary.
pub fn run_command(cmd: &str, args: &[&str]) -> Result<(), AppError> {
    if is_root() {
        let status = Command::new(cmd)
            .args(args)
            .status()
            .map_err(|e| AppError::apply_failed(format!("Failed to spawn {cmd}: {e}")))?;
        if !status.success() {
            return Err(AppError::apply_failed(format!(
                "Command {cmd} exited with non-zero status"
            )));
        }
        return Ok(());
    }
    elevate(cmd, args)
}

/// Low-level elevation primitive.
/// Payload is passed via argv arrays only — never shell interpolation.
pub fn elevate(cmd: &str, args: &[&str]) -> Result<(), AppError> {
    let tool = detect_elevation_tool()
        .ok_or_else(|| AppError::elevation_failed("No elevation tool found (sudo/pkexec/doas)"))?;

    let status = Command::new(tool)
        .arg(cmd)
        .args(args)
        .status()
        .map_err(|e| AppError::elevation_failed(format!("Failed to spawn elevation: {e}")))?;

    if !status.success() {
        return Err(AppError::elevation_failed(
            "Elevation refused or command failed",
        ));
    }

    Ok(())
}

/// Return `~/.local/share/flux/.tmp/`, creating it if necessary.
fn flux_tmp_dir() -> Result<PathBuf, AppError> {
    let dir = dirs::home_dir()
        .ok_or_else(|| AppError::apply_failed("Cannot determine home directory"))?
        .join(".local/share/flux/.tmp");
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::apply_failed(format!("Failed to create tmp dir: {e}")))?;
    Ok(dir)
}

fn which(cmd: &str) -> Option<PathBuf> {
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
    fn test_detect_elevation_tool_returns_something_or_none() {
        // In CI we usually have sudo; on minimal containers we may not.
        // Just ensure it doesn't panic.
        let _ = detect_elevation_tool();
    }

    #[test]
    fn test_is_root_matches_geteuid() {
        assert_eq!(is_root(), nix::unistd::geteuid().is_root());
    }

    #[test]
    fn test_write_file_when_root() {
        if !is_root() {
            return;
        }
        let path = "/tmp/flux-priv-test-write-file";
        let content = "nameserver 1.1.1.1\n";
        write_file(path, content).unwrap();
        let read_back = std::fs::read_to_string(path).unwrap();
        assert_eq!(read_back, content);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_create_dir_all_when_root() {
        if !is_root() {
            return;
        }
        let path = "/tmp/flux-priv-test-dir/nested";
        create_dir_all(path).unwrap();
        assert!(std::path::Path::new(path).is_dir());
        let _ = std::fs::remove_dir_all("/tmp/flux-priv-test-dir");
    }

    #[test]
    fn test_remove_file_when_root() {
        if !is_root() {
            return;
        }
        let path = "/tmp/flux-priv-test-rm";
        std::fs::write(path, "x").unwrap();
        remove_file(path).unwrap();
        assert!(!std::path::Path::new(path).exists());
    }

    #[test]
    fn test_run_command_when_root() {
        if !is_root() {
            return;
        }
        // Use `true` which exits 0 everywhere.
        run_command("true", &[]).unwrap();
    }

    #[test]
    fn test_run_command_failure_when_root() {
        if !is_root() {
            return;
        }
        // Use `false` which exits 1 everywhere.
        let result = run_command("false", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_elevate_refused_when_no_tool() {
        if is_root() {
            // Root path bypasses elevation entirely.
            return;
        }
        // We can't easily mock detect_elevation_tool(), but we can at least
        // verify the error message when no tool is present by testing the
        // public functions on a non-root system. On CI there is usually sudo,
        // so this test would pass via the real tool. Skip in that case.
        if detect_elevation_tool().is_some() {
            return;
        }
        let result = elevate("true", &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("No elevation tool found"));
    }
}
