# Post-Quantum Cryptography (PQC) Migration Plan

## Status

v0.1.0 uses system TLS stacks for DoH/DoT/DoQ. No direct TLS dependency in Flux itself.

## v0.2+ Plan

For `dns update-registry` (fetchable provider registry), Flux will use **rustls** with PQC-hybrid support:

- **KEM:** `X25519MLKEM768` (hybrid X25519 + ML-KEM-768)
- **Signatures:** `ML-DSA-65` where supported by the rustls release

## Rationale

- NIST FIPS 203 (ML-KEM) and FIPS 204 (ML-DSA) are the PQC standards
- rustls supports PQC-hybrid via the `aws-lc-rs` or `ring` crypto providers
- User-initiated fetches only — consistent with PFA policy (no automatic background calls)

## Dependencies

- `rustls` with `X25519MLKEM768` feature
- `rustls-pki-types` for certificate handling

## Timeline

| Version | Milestone |
|---------|-----------|
| v0.1.0 | No PQC (system TLS only) |
| v0.2.0 | `update-registry` with rustls + PQC-hybrid |
| v0.3.0 | Runtime DNSCrypt stamp fetching with PQC-aware transport |

*--- Forged in Steelbore ---*
