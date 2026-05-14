// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// Generic JSON response envelope: `{ metadata, data }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub metadata: Metadata,
    pub data: T,
}

impl<T> Response<T> {
    pub fn new(data: T) -> Self {
        Self {
            metadata: Metadata::default(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub tool: String,
    pub version: String,
    pub command: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoking_agent: Option<String>,
    pub maintainer: String,
    pub website: String,
}

impl Default for Metadata {
    fn default() -> Self {
        let invoking_agent = detect_invoking_agent();
        Self {
            tool: "flux".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: std::env::args().collect::<Vec<_>>().join(" "),
            timestamp: super::now_utc(),
            pagination: None,
            invoking_agent,
            maintainer: "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>".to_string(),
            website: "https://Flux.SpacecraftSoftware.org/".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

fn detect_invoking_agent() -> Option<String> {
    if std::env::var("CLAUDECODE").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true")) {
        Some("claude-code".to_string())
    } else if std::env::var("CURSOR_AGENT")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        Some("cursor".to_string())
    } else if std::env::var("GEMINI_CLI").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        Some("gemini-cli".to_string())
    } else {
        None
    }
}
