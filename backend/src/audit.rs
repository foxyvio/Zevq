use crate::{
    parser::{parse_rust_source, Finding, FindingKind, RiskLevel},
    solver::{report_to_json, verify_findings, SolverError, VerificationReport},
};

const MAX_PAYLOAD_BYTES: usize = 20_000;

pub fn sanitize_payload(payload: &str) -> String {
    payload
        .chars()
        .filter(|character| !character.is_control() || character.is_whitespace())
        .take(MAX_PAYLOAD_BYTES)
        .collect()
}

pub fn audit_payload(payload: &str) -> VerificationReport {
    let sanitized = sanitize_payload(payload);
    let mut findings = parse_rust_source(&sanitized);

    if contains_division_minus_pattern(&sanitized)
        && !findings
            .iter()
            .any(|finding| finding.kind == FindingKind::UnboundedArithmetic)
    {
        findings.push(Finding {
            kind: FindingKind::UnboundedArithmetic,
            risk: RiskLevel::Critical,
            evidence: "division over subtraction expression".to_string(),
            constraint: "denominator == 0".to_string(),
        });
    }

    verify_findings(findings)
}

pub fn audit_payload_json(payload: &str) -> Result<String, SolverError> {
    report_to_json(&audit_payload(payload))
}

fn contains_division_minus_pattern(payload: &str) -> bool {
    let bytes = payload.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'/' {
            let mut cursor = index + 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && bytes[cursor] == b'(' {
                let window_end = (cursor + 96).min(bytes.len());
                if bytes[cursor..window_end].contains(&b'-') {
                    return true;
                }
            }
        }
        index += 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::VerificationStatus;

    #[test]
    fn rust_audit_facade_emits_crash_for_division_minus() {
        let report = audit_payload("score / (z - 5)");
        assert_eq!(report.status, VerificationStatus::Crash);
        assert!(report
            .counter_examples
            .iter()
            .any(|example| example.variable == "z" && example.value == "5"));
    }

    #[test]
    fn sanitizes_control_characters() {
        assert_eq!(sanitize_payload("safe\u{0000}\ntext"), "safe\ntext");
    }
}
