#!/usr/bin/env bash
set -euo pipefail

cargo fmt --manifest-path backend/Cargo.toml --check
cargo test --manifest-path backend/Cargo.toml
cargo run --manifest-path backend/Cargo.toml -- "$(cat samples/confident_ai.json)" >/tmp/zevq_assessment_smoke.json
