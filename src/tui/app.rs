// SPDX-License-Identifier: GPL-3.0-or-later

use crate::detection::Backend;
use crate::registry::{Protocol, Tier, get_provider, list_providers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    ProviderSelect,
    TierSelect,
    ProtocolSelect,
    Confirmation,
    #[allow(dead_code)]
    Progress,
    Status,
    About,
}

#[derive(Debug, Clone)]
pub struct App {
    pub screen: Screen,
    pub selected_provider: Option<String>,
    pub selected_tier: Option<Tier>,
    pub selected_protocol: Option<Protocol>,
    pub detected_backend: Option<Backend>,
    pub menu_idx: usize,
    pub provider_idx: usize,
    pub tier_idx: usize,
    pub protocol_idx: usize,
    pub confirm: bool,
    #[allow(dead_code)]
    pub ntp: bool,
    #[allow(dead_code)]
    pub vpn: Option<String>,
    #[allow(dead_code)]
    pub ipv4_only: bool,
    #[allow(dead_code)]
    pub ipv4_only_idx: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let backend = crate::detection::detect_backend().ok();
        Self {
            screen: Screen::MainMenu,
            selected_provider: None,
            selected_tier: None,
            selected_protocol: None,
            detected_backend: backend,
            menu_idx: 0,
            provider_idx: 0,
            tier_idx: 0,
            protocol_idx: 0,
            confirm: false,
            ntp: false,
            vpn: None,
            ipv4_only: false,
            ipv4_only_idx: 0,
            status_message: None,
        }
    }

    pub fn menu_items() -> &'static [&'static str] {
        &[
            "Apply DNS",
            "Configure NTP",
            "VPN",
            "Status",
            "Restore",
            "Detect",
            "About",
            "Quit",
        ]
    }

    pub fn provider_items() -> Vec<String> {
        list_providers()
            .into_iter()
            .map(|p| p.name.clone())
            .collect()
    }

    pub fn tier_items(&self) -> Vec<String> {
        let slug = self.selected_provider.as_deref().unwrap_or("cloudflare");
        get_provider(slug)
            .map(|p| p.tiers.iter().map(|t| t.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn protocol_items(&self) -> Vec<String> {
        let slug = self.selected_provider.as_deref().unwrap_or("cloudflare");
        let provider = match get_provider(slug) {
            Some(p) => p,
            None => return Vec::new(),
        };
        let tier = self.selected_tier;
        let backend = self.detected_backend.unwrap_or(Backend::ResolvConf);
        crate::registry::valid_protocols(provider, tier, backend)
            .iter()
            .map(|p| p.to_string().to_uppercase())
            .collect()
    }

    pub fn apply_selection(&mut self) -> Result<String, crate::error::AppError> {
        let provider = self.selected_provider.clone().unwrap_or_default();
        let tier = self.selected_tier;
        let protocol = self
            .selected_protocol
            .ok_or_else(|| crate::error::AppError::usage_error("No protocol selected"))?;

        let mode = crate::output::mode::OutputMode::from_args(
            None, false, false, None, false, false, false, true, None, false, false,
        );

        crate::orchestrator::run_apply(
            &provider,
            tier,
            protocol,
            self.ipv4_only,
            false,
            false,
            false,
            self.ntp,
            None,
            &mode,
        )?;

        Ok(format!(
            "Applied {} {} {}",
            provider,
            tier.map(|t| t.to_string()).unwrap_or_default(),
            protocol
        ))
    }
}
