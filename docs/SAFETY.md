# Safety Justifications

## Registry

As of v0.1.0, Flux contains **zero** `unsafe` blocks.

All operations are memory-safe Rust. No FFI, no raw pointer dereferences, no manual memory management.

## Policy

If an `unsafe` block is ever required:

1. Inline safety justification comment required
2. Entry required in this file
3. Entry must document: what, why, and why safe

*--- Forged in Spacecraft Software ---*
