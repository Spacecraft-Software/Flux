// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical exit codes per SFRS §4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExitCode {
    Success = 0,
    GeneralFailure = 1,
    UsageError = 2,
    DetectionFailed = 3,
    ElevationFailed = 4,
    Conflict = 5,
    ApplyFailed = 6,
    VerificationFailed = 7,
    VpnError = 8,
    RegistryFetchFailed = 9,
}

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

/// Structured application error with tips-thinking hint.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{message}")]
pub struct AppError {
    pub code: String,
    pub exit_code: i32,
    pub message: String,
    pub hint: String,
    pub timestamp: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

impl AppError {
    pub fn new(code: ExitCode, message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            code: format!("{code:?}"),
            exit_code: code.as_i32(),
            message: message.into(),
            hint: hint.into(),
            timestamp: crate::output::now_utc(),
            command: std::env::args().collect::<Vec<_>>().join(" "),
            docs_url: Some(format!("https://Flux.Steelbore.com/errors/{code:?}")),
        }
    }

    pub fn detection_failed(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::DetectionFailed, msg, "dns detect --json")
    }

    #[allow(dead_code)]
    pub fn elevation_failed(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::ElevationFailed, msg, "sudo dns apply ...")
    }

    #[allow(dead_code)]
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::Conflict, msg, "dns status --json")
    }

    pub fn apply_failed(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::ApplyFailed, msg, "dns restore && dns detect")
    }

    pub fn verification_failed(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::VerificationFailed, msg, "dns restore")
    }

    pub fn vpn_error(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::VpnError, msg, "dns vpn status --json")
    }

    pub fn usage_error(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::UsageError, msg, "dns --help")
    }

    pub fn general(msg: impl Into<String>) -> Self {
        Self::new(ExitCode::GeneralFailure, msg, "dns --help")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::GeneralFailure.as_i32(), 1);
        assert_eq!(ExitCode::UsageError.as_i32(), 2);
        assert_eq!(ExitCode::DetectionFailed.as_i32(), 3);
        assert_eq!(ExitCode::ElevationFailed.as_i32(), 4);
        assert_eq!(ExitCode::Conflict.as_i32(), 5);
        assert_eq!(ExitCode::ApplyFailed.as_i32(), 6);
        assert_eq!(ExitCode::VerificationFailed.as_i32(), 7);
        assert_eq!(ExitCode::VpnError.as_i32(), 8);
        assert_eq!(ExitCode::RegistryFetchFailed.as_i32(), 9);
    }

    #[test]
    fn test_app_error_hint_runnable() {
        let err = AppError::detection_failed("test");
        assert_eq!(err.exit_code, 3);
        assert_eq!(err.hint, "dns detect --json");
        assert!(err.timestamp.ends_with('Z'));
    }

    #[test]
    fn test_app_error_serde_roundtrip() {
        let err = AppError::usage_error("bad arg");
        let json = serde_json::to_string(&err).unwrap();
        let decoded: AppError = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.code, err.code);
        assert_eq!(decoded.exit_code, err.exit_code);
        assert_eq!(decoded.message, err.message);
        assert_eq!(decoded.hint, err.hint);
    }
}
