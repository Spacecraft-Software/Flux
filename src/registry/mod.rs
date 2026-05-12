// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

pub mod providers;

/// DNS transport protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Plain,
    #[serde(rename = "dot")]
    DoT,
    #[serde(rename = "doh")]
    DoH,
    #[serde(rename = "doq")]
    DoQ,
    DnsCrypt,
    Warp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Protocol::Plain => "plain",
            Protocol::DoT => "dot",
            Protocol::DoH => "doh",
            Protocol::DoQ => "doq",
            Protocol::DnsCrypt => "dnscrypt",
            Protocol::Warp => "warp",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for Protocol {
    type Err = crate::error::AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Protocol::Plain),
            "dot" => Ok(Protocol::DoT),
            "doh" => Ok(Protocol::DoH),
            "doq" => Ok(Protocol::DoQ),
            "dnscrypt" => Ok(Protocol::DnsCrypt),
            "warp" => Ok(Protocol::Warp),
            _ => Err(crate::error::AppError::usage_error(format!(
                "Unknown protocol: {s}. Valid: plain, dot, doh, doq, dnscrypt, warp"
            ))),
        }
    }
}

/// Filtering tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Standard,
    Malware,
    Family,
    Unfiltered,
    Ecs,
    Unsecured,
    Secured,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Tier::Standard => "standard",
            Tier::Malware => "malware",
            Tier::Family => "family",
            Tier::Unfiltered => "unfiltered",
            Tier::Ecs => "ecs",
            Tier::Unsecured => "unsecured",
            Tier::Secured => "secured",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for Tier {
    type Err = crate::error::AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(Tier::Standard),
            "malware" => Ok(Tier::Malware),
            "family" => Ok(Tier::Family),
            "unfiltered" => Ok(Tier::Unfiltered),
            "ecs" => Ok(Tier::Ecs),
            "unsecured" => Ok(Tier::Unsecured),
            "secured" => Ok(Tier::Secured),
            _ => Err(crate::error::AppError::usage_error(format!(
                "Unknown tier: {s}. Valid: standard, malware, family, unfiltered, ecs, unsecured, secured"
            ))),
        }
    }
}

/// Address and endpoint information for a provider tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAddresses {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_secondary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_secondary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dot_host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doh_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doq_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dnscrypt_stamp: Option<String>,
}

/// A DNS provider entry in the compile-time registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub slug: String,
    pub name: String,
    pub tiers: Vec<Tier>,
    pub protocols: Vec<Protocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntp_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Per-tier address configuration.
    pub addresses: std::collections::HashMap<Tier, ProviderAddresses>,
}

/// Get a provider by slug.
pub fn get_provider(slug: &str) -> Option<&Provider> {
    std::sync::LazyLock::force(&providers::REGISTRY)
        .iter()
        .find(|p| p.slug == slug)
}

/// List all providers.
pub fn list_providers() -> Vec<&'static Provider> {
    std::sync::LazyLock::force(&providers::REGISTRY)
        .iter()
        .collect()
}

/// Return valid protocols for a provider/tier/backend combination.
pub fn valid_protocols(
    provider: &Provider,
    _tier: Option<Tier>,
    backend: crate::detection::Backend,
) -> Vec<Protocol> {
    let mut protos = provider.protocols.clone();
    // Filter by backend constraints per PRD §7.4
    protos.retain(|p| backend_supports_protocol(backend, *p));
    protos
}

/// Validate that a provider/tier/protocol combination is supported.
pub fn validate_combination(
    provider: &Provider,
    tier: Option<Tier>,
    protocol: Protocol,
) -> Result<(), crate::error::AppError> {
    if !provider.protocols.contains(&protocol) {
        return Err(crate::error::AppError::usage_error(format!(
            "Provider '{}' does not support protocol '{}'",
            provider.slug, protocol
        )));
    }
    if let Some(t) = tier {
        if !provider.tiers.contains(&t) {
            return Err(crate::error::AppError::usage_error(format!(
                "Provider '{}' does not have tier '{}'",
                provider.slug, t
            )));
        }
        // Check tier-specific protocol support via addresses
        if let Some(addrs) = provider.addresses.get(&t) {
            let supported = match protocol {
                Protocol::Plain => addrs.ipv4_primary.is_some() || addrs.ipv6_primary.is_some(),
                Protocol::DoT => addrs.dot_host.is_some(),
                Protocol::DoH => addrs.doh_url.is_some(),
                Protocol::DoQ => addrs.doq_url.is_some(),
                Protocol::DnsCrypt => addrs.dnscrypt_stamp.is_some(),
                Protocol::Warp => true,
            };
            if !supported {
                return Err(crate::error::AppError::usage_error(format!(
                    "Tier '{}' on provider '{}' lacks addresses for protocol '{}'",
                    t, provider.slug, protocol
                )));
            }
        }
    }
    Ok(())
}

pub(crate) fn backend_supports_protocol(
    backend: crate::detection::Backend,
    protocol: Protocol,
) -> bool {
    use crate::detection::Backend;
    match (backend, protocol) {
        (Backend::ResolvConf, Protocol::Plain) => true,
        (Backend::SystemdResolved, Protocol::Plain)
        | (Backend::SystemdResolved, Protocol::DoT)
        | (Backend::SystemdResolved, Protocol::DoH) => true,
        (Backend::NetworkManager, Protocol::Plain) => true,
        (Backend::Resolvectl, Protocol::Plain) | (Backend::Resolvectl, Protocol::DoT) => true,
        (Backend::Nixos, Protocol::Plain)
        | (Backend::Nixos, Protocol::DoT)
        | (Backend::Nixos, Protocol::DoH) => true,
        (Backend::FreeBsd | Backend::OpenBsd | Backend::NetBsd, Protocol::Plain)
        | (Backend::FreeBsd | Backend::OpenBsd | Backend::NetBsd, Protocol::DoT)
        | (Backend::FreeBsd | Backend::OpenBsd | Backend::NetBsd, Protocol::DoH) => true,
        (Backend::Stub, Protocol::DoQ) | (Backend::Stub, Protocol::DnsCrypt) => true,
        (_, Protocol::DoQ) | (_, Protocol::DnsCrypt) => false, // requires local stub
        (_, Protocol::Warp) => true,                           // warp-cli handles itself
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_provider_google() {
        let p = get_provider("google").expect("google should exist");
        assert_eq!(p.slug, "google");
        assert!(p.tiers.contains(&Tier::Standard));
        assert!(p.protocols.contains(&Protocol::DoH));
        assert!(!p.protocols.contains(&Protocol::DoQ));
    }

    #[test]
    fn test_get_provider_cloudflare() {
        let p = get_provider("cloudflare").expect("cloudflare should exist");
        assert_eq!(p.slug, "cloudflare");
        assert!(p.tiers.contains(&Tier::Family));
        assert!(p.ntp_server.is_some());
    }

    #[test]
    fn test_get_provider_adguard() {
        let p = get_provider("adguard").expect("adguard should exist");
        assert!(p.protocols.contains(&Protocol::DoQ));
        assert!(p.protocols.contains(&Protocol::DnsCrypt));
    }

    #[test]
    fn test_list_providers_count() {
        let providers = list_providers();
        assert_eq!(providers.len(), 5);
    }

    #[test]
    fn test_validate_combination_ok() {
        let p = get_provider("cloudflare").unwrap();
        assert!(validate_combination(p, Some(Tier::Family), Protocol::DoH).is_ok());
    }

    #[test]
    fn test_validate_combination_bad_protocol() {
        let p = get_provider("google").unwrap();
        assert!(validate_combination(p, Some(Tier::Standard), Protocol::DoQ).is_err());
    }

    #[test]
    fn test_validate_combination_bad_tier() {
        let p = get_provider("google").unwrap();
        assert!(validate_combination(p, Some(Tier::Family), Protocol::Plain).is_err());
    }

    #[test]
    fn test_protocol_parse() {
        assert_eq!("dot".parse::<Protocol>().unwrap(), Protocol::DoT);
        assert!("invalid".parse::<Protocol>().is_err());
    }

    #[test]
    fn test_tier_parse() {
        assert_eq!("family".parse::<Tier>().unwrap(), Tier::Family);
        assert!("invalid".parse::<Tier>().is_err());
    }

    #[test]
    fn test_backend_supports_protocol() {
        use crate::detection::Backend;
        assert!(backend_supports_protocol(
            Backend::SystemdResolved,
            Protocol::DoH
        ));
        assert!(!backend_supports_protocol(
            Backend::ResolvConf,
            Protocol::DoT
        ));
        assert!(!backend_supports_protocol(
            Backend::NetworkManager,
            Protocol::DoQ
        ));
    }
}
