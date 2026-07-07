# Zevq AI

Zevq AI is an enterprise AI safety and certification prototype that combines a Rust-first verification core, a Next.js telemetry portal, and a Flutter-ready local shell.

## Architecture ratios

- **Rust core (50%+)**: AST parsing, vulnerability extraction, SMT/Z3 solver checks, async verification orchestration, and FFI-safe report shapes.
- **Next.js + TypeScript (30%)**: enterprise audit API and dashboard for high-density vulnerability reporting.
- **Flutter + Dart (20%)**: cross-platform wrapper scaffold for local or air-gapped execution through a native Rust bridge.

## Repository map

- `backend/`: Rust verification engine.
  - `src/parser.rs`: `syn` visitor that extracts arithmetic, SQL string-concat, and tool-authorization findings.
  - `src/solver.rs`: Z3-backed bounded checks that turn parsed findings into counter-example reports.
  - `src/main.rs`: Tokio API entrypoint with enum-based errors.
  - `src/ffi.rs`: C ABI bridge that exposes a JSON audit report to Flutter or other native hosts.
- `pages/api/audit.ts`: Wizard-of-Oz audit proxy that sanitizes inbound text and emits deterministic crash reports.
- `components/Dashboard.tsx`: matte white/orange/pink enterprise dashboard.
- `zevq_desktop_mobile/`: Flutter shell scaffold with a Rust bridge boundary.
