# Zevq AI

Zevq AI is now a Rust + Flutter safety-certification prototype. The web/Next.js layer has been removed so all executable safety logic lives in Rust and all operator-facing UI is delivered through Flutter for desktop, mobile, and air-gapped deployments.

## Architecture allocation

- **Rust core (70%+)**: AST parsing, vulnerability extraction, SMT/Z3 solver checks, Wizard-of-Oz audit emulation, async orchestration, and FFI-safe report serialization.
- **Flutter + Dart shell (30%)**: localized enterprise dashboard and native bridge boundary for closed-network execution.

## Repository map

- `backend/`: Rust verification engine.
  - `src/parser.rs`: `syn` visitor that extracts arithmetic, SQL string-concat, and tool-authorization findings.
  - `src/solver.rs`: Z3-backed bounded checks that turn parsed findings into counter-example reports.
  - `src/audit.rs`: Rust-native audit facade replacing the old Next.js API route; sanitizes payloads and emits deterministic crash reports.
  - `src/main.rs`: Tokio CLI entrypoint with enum-based errors.
  - `src/ffi.rs`: C ABI bridge that exposes JSON audit reports to Flutter or other native hosts.
- `zevq_desktop_mobile/`: Flutter shell that calls the Rust cdylib directly through Dart FFI.
