# Zevq AI

Zevq AI is a Rust + Flutter mathematical rating engine for AI startups. It is designed as a local, air-gapped "Moody's for AI safety" prototype: Rust performs deterministic startup assessment math, while Flutter presents the operator dashboard.

## What it does now

- Scores an AI startup profile across reliability, safety, and governance.
- Uses explicit formulas for claim-vs-evaluation gaps, adversarial resilience, incident penalties, latency penalties, and human-override controls.
- Emits a rating ladder: `Aaa`, `Aa`, `A`, `Bbb`, `Bb`, `B`, or `C`.
- Returns a verdict: `CERTIFY`, `CONDITIONAL`, or `REJECT_FOR_NOW`.
- Produces scenario-level proofs and remediation actions for every failed or watchlisted mathematical control.
- Exposes a C ABI bridge so Flutter can call the Rust engine offline.

## Production-readiness hardening

- Versioned report schema via `schema_version` and `engine_version`.
- Input validation for malformed JSON, missing names, invalid percentages, and invalid latency values.
- FFI-safe error JSON instead of panics for bad native input.
- Local production check script for Rust formatting, Rust tests, and CLI smoke tests (`scripts/check.sh`).
- Sample evidence profile in `samples/confident_ai.json`.

## Repository map

- `backend/src/assessment.rs`: mathematical assessment engine, validation, scenarios, scoring formulas, ratings, remediation plans, and tests.
- `backend/src/main.rs`: Tokio CLI entrypoint for local batch assessment.
- `backend/src/ffi.rs`: native bridge exported to Flutter as `zevq_assess_startup` with the legacy `zevq_audit_source` alias.
- `zevq_desktop_mobile/lib/main.dart`: Flutter operator dashboard for startup profile input and rating output.
- `zevq_desktop_mobile/lib/rust_bridge.dart`: Dart FFI wrapper around the Rust shared library with user-visible engine errors.

## Example CLI

```bash
cargo run --manifest-path backend/Cargo.toml -- "$(cat samples/confident_ai.json)"
```

The default benchmark intentionally grades weak evidence harshly so strong AI startups must prove their claims mathematically instead of marketing their way through certification.
