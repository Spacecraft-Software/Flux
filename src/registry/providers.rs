// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::LazyLock;

use super::{Provider, ProviderAddresses, Tier};

/// Compile-time provider registry.
pub static REGISTRY: LazyLock<Vec<Provider>> = LazyLock::new(|| {
    vec![
        Provider {
            slug: "google".to_string(),
            name: "Google Public DNS".to_string(),
            tiers: vec![Tier::Standard],
            protocols: vec![
                super::Protocol::Plain,
                super::Protocol::DoT,
                super::Protocol::DoH,
            ],
            ntp_server: Some("time.google.com".to_string()),
            notes: None,
            addresses: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    Tier::Standard,
                    ProviderAddresses {
                        ipv4_primary: Some("8.8.8.8".to_string()),
                        ipv4_secondary: Some("8.8.4.4".to_string()),
                        ipv6_primary: Some("2001:4860:4860::8888".to_string()),
                        ipv6_secondary: Some("2001:4860:4860::8844".to_string()),
                        dot_host: Some("dns.google".to_string()),
                        doh_url: Some("https://dns.google/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: None,
                    },
                );
                m
            },
        },
        Provider {
            slug: "cloudflare".to_string(),
            name: "Cloudflare DNS".to_string(),
            tiers: vec![Tier::Standard, Tier::Malware, Tier::Family],
            protocols: vec![
                super::Protocol::Plain,
                super::Protocol::DoT,
                super::Protocol::DoH,
            ],
            ntp_server: Some("time.cloudflare.com".to_string()),
            notes: Some("WARP integration available".to_string()),
            addresses: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    Tier::Standard,
                    ProviderAddresses {
                        ipv4_primary: Some("1.1.1.1".to_string()),
                        ipv4_secondary: Some("1.0.0.1".to_string()),
                        ipv6_primary: Some("2606:4700:4700::1111".to_string()),
                        ipv6_secondary: Some("2606:4700:4700::1001".to_string()),
                        dot_host: Some("one.one.one.one".to_string()),
                        doh_url: Some("https://cloudflare-dns.com/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: None,
                    },
                );
                m.insert(
                    Tier::Malware,
                    ProviderAddresses {
                        ipv4_primary: Some("1.1.1.2".to_string()),
                        ipv4_secondary: Some("1.0.0.2".to_string()),
                        ipv6_primary: Some("2606:4700:4700::1112".to_string()),
                        ipv6_secondary: Some("2606:4700:4700::1002".to_string()),
                        dot_host: Some("security.cloudflare-dns.com".to_string()),
                        doh_url: Some("https://security.cloudflare-dns.com/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: None,
                    },
                );
                m.insert(
                    Tier::Family,
                    ProviderAddresses {
                        ipv4_primary: Some("1.1.1.3".to_string()),
                        ipv4_secondary: Some("1.0.0.3".to_string()),
                        ipv6_primary: Some("2606:4700:4700::1113".to_string()),
                        ipv6_secondary: Some("2606:4700:4700::1003".to_string()),
                        dot_host: Some("family.cloudflare-dns.com".to_string()),
                        doh_url: Some("https://family.cloudflare-dns.com/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: None,
                    },
                );
                m
            },
        },
        Provider {
            slug: "adguard".to_string(),
            name: "AdGuard DNS".to_string(),
            tiers: vec![Tier::Standard, Tier::Family, Tier::Unfiltered],
            protocols: vec![
                super::Protocol::Plain,
                super::Protocol::DoT,
                super::Protocol::DoH,
                super::Protocol::DoQ,
                super::Protocol::DnsCrypt,
            ],
            ntp_server: None,
            notes: Some("All six protocol families supported".to_string()),
            addresses: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    Tier::Standard,
                    ProviderAddresses {
                        ipv4_primary: Some("94.140.14.14".to_string()),
                        ipv4_secondary: Some("94.140.15.15".to_string()),
                        ipv6_primary: Some("2a10:50c0::ad1:ff".to_string()),
                        ipv6_secondary: Some("2a10:50c0::ad2:ff".to_string()),
                        dot_host: Some("dns.adguard-dns.com".to_string()),
                        doh_url: Some("https://dns.adguard-dns.com/dns-query".to_string()),
                        doq_url: Some("quic://dns.adguard-dns.com".to_string()),
                        dnscrypt_stamp: Some(
                            "sdns://AQMAAAAAAAAAETk0LjE0MC4xNC4xNDo1NDQzILgVFiDLF3SHe7CO-5C8EWaTfW5XlT7C8iEwqc0O2QYfGDIuZG5zY3J5cHQtY2VydC5kbnMuYWRndWFyZC1kbnMuY29t".to_string(),
                        ),
                    },
                );
                m.insert(
                    Tier::Family,
                    ProviderAddresses {
                        ipv4_primary: Some("94.140.14.15".to_string()),
                        ipv4_secondary: Some("94.140.15.16".to_string()),
                        ipv6_primary: Some("2a10:50c0::bad1:ff".to_string()),
                        ipv6_secondary: Some("2a10:50c0::bad2:ff".to_string()),
                        dot_host: Some("family.adguard-dns.com".to_string()),
                        doh_url: Some("https://family.adguard-dns.com/dns-query".to_string()),
                        doq_url: Some("quic://family.adguard-dns.com".to_string()),
                        dnscrypt_stamp: Some(
                            "sdns://AQMAAAAAAAAAETk0LjE0MC4xNC4xNTo1NDQzILgVFiDLF3SHe7CO-5C8EWaTfW5XlT7C8iEwqc0O2QYfGDIuZG5zY3J5cHQtY2VydC5kbnMuYWRndWFyZC1kbnMuY29t".to_string(),
                        ),
                    },
                );
                m.insert(
                    Tier::Unfiltered,
                    ProviderAddresses {
                        ipv4_primary: Some("94.140.14.140".to_string()),
                        ipv4_secondary: Some("94.140.14.141".to_string()),
                        ipv6_primary: Some("2a10:50c0::1:ff".to_string()),
                        ipv6_secondary: Some("2a10:50c0::2:ff".to_string()),
                        dot_host: Some("unfiltered.adguard-dns.com".to_string()),
                        doh_url: Some("https://unfiltered.adguard-dns.com/dns-query".to_string()),
                        doq_url: Some("quic://unfiltered.adguard-dns.com".to_string()),
                        dnscrypt_stamp: Some(
                            "sdns://AQMAAAAAAAAAETk0LjE0MC4xNC4xNDA6NTQ0MyC4FRYgyxd0h3uwjvsQvBFmk31uV5U+wvIhMKnNDtkGHxgyLmRuc2NyeXB0LWNlcnQuZG5zLmFkZ3VhcmQtZG5zLmNvbQ".to_string(),
                        ),
                    },
                );
                m
            },
        },
        Provider {
            slug: "quad9".to_string(),
            name: "Quad9".to_string(),
            tiers: vec![Tier::Secured, Tier::Ecs, Tier::Unsecured],
            protocols: vec![
                super::Protocol::Plain,
                super::Protocol::DoT,
                super::Protocol::DoH,
                super::Protocol::DnsCrypt,
            ],
            ntp_server: None,
            notes: None,
            addresses: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    Tier::Secured,
                    ProviderAddresses {
                        ipv4_primary: Some("9.9.9.9".to_string()),
                        ipv4_secondary: Some("149.112.112.112".to_string()),
                        ipv6_primary: Some("2620:fe::fe".to_string()),
                        ipv6_secondary: Some("2620:fe::9".to_string()),
                        dot_host: Some("dns.quad9.net".to_string()),
                        doh_url: Some("https://dns.quad9.net/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: Some(
                            "sdns://AQYAAAAAAAAADTkuOS45Ljk6ODQ0MyCqQzH-F00DPxNtG3S2F-dLxU9L65LPP7RMKsx3Nqa2JyEyLmRuc2NyeXB0LWNlcnQuZGlnaWNlcnQuY29t".to_string(),
                        ),
                    },
                );
                m.insert(
                    Tier::Ecs,
                    ProviderAddresses {
                        ipv4_primary: Some("9.9.9.11".to_string()),
                        ipv4_secondary: Some("149.112.112.11".to_string()),
                        ipv6_primary: Some("2620:fe::11".to_string()),
                        ipv6_secondary: Some("2620:fe::fe:11".to_string()),
                        dot_host: Some("dns11.quad9.net".to_string()),
                        doh_url: Some("https://dns11.quad9.net/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: Some(
                            "sdns://AQYAAAAAAAAADjkuOS45LjExOjg0NDMgqCM0IdFJbGgBwhXjbsbmLgMQaNXnQcjJrNB6yR6KxWMyLmRuc2NyeXB0LWNlcnQuZGlnaWNlcnQuY29t".to_string(),
                        ),
                    },
                );
                m.insert(
                    Tier::Unsecured,
                    ProviderAddresses {
                        ipv4_primary: Some("9.9.9.10".to_string()),
                        ipv4_secondary: Some("149.112.112.10".to_string()),
                        ipv6_primary: Some("2620:fe::10".to_string()),
                        ipv6_secondary: Some("2620:fe::fe:10".to_string()),
                        dot_host: Some("dns10.quad9.net".to_string()),
                        doh_url: Some("https://dns10.quad9.net/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: Some(
                            "sdns://AQYAAAAAAAAADjkuOS45LjEwOjg0NDMgwDTDqFVd1AAJdF1hwQdrlOz7a1-z7yuzmR51W2THJ1MyLmRuc2NyeXB0LWNlcnQuZGlnaWNlcnQuY29t".to_string(),
                        ),
                    },
                );
                m
            },
        },
        Provider {
            slug: "opendns".to_string(),
            name: "OpenDNS".to_string(),
            tiers: vec![Tier::Standard, Tier::Family],
            protocols: vec![super::Protocol::Plain, super::Protocol::DoH],
            ntp_server: None,
            notes: None,
            addresses: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    Tier::Standard,
                    ProviderAddresses {
                        ipv4_primary: Some("208.67.222.222".to_string()),
                        ipv4_secondary: Some("208.67.220.220".to_string()),
                        ipv6_primary: Some("2620:119:35::35".to_string()),
                        ipv6_secondary: Some("2620:119:53::53".to_string()),
                        dot_host: None,
                        doh_url: Some("https://doh.opendns.com/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: None,
                    },
                );
                m.insert(
                    Tier::Family,
                    ProviderAddresses {
                        ipv4_primary: Some("208.67.222.123".to_string()),
                        ipv4_secondary: Some("208.67.220.123".to_string()),
                        ipv6_primary: Some("2620:119:35::123".to_string()),
                        ipv6_secondary: Some("2620:119:53::123".to_string()),
                        dot_host: None,
                        doh_url: Some("https://doh.familyshield.opendns.com/dns-query".to_string()),
                        doq_url: None,
                        dnscrypt_stamp: None,
                    },
                );
                m
            },
        },
    ]
});
