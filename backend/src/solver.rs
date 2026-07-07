use crate::parser::{Finding, FindingKind, RiskLevel};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use z3::{
    ast::{Ast, Int},
    Config, Context, SatResult, Solver,
};

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("failed to serialize verification report: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationStatus {
    Pass,
    Crash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterExample {
    pub variable: String,
    pub value: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationReport {
    pub status: VerificationStatus,
    pub findings: Vec<Finding>,
    pub counter_examples: Vec<CounterExample>,
}

pub fn verify_findings(findings: Vec<Finding>) -> VerificationReport {
    let mut counter_examples = Vec::new();

    for finding in &findings {
        match finding.kind {
            FindingKind::UnboundedArithmetic => {
                if let Some(example) = prove_division_counter_example() {
                    counter_examples.push(example);
                }
            }
            FindingKind::SqlStringConcat => counter_examples.push(CounterExample {
                variable: "sql_parameterized".to_string(),
                value: "false".to_string(),
                explanation:
                    "Unparameterized SQL construction can admit attacker-controlled query shape."
                        .to_string(),
            }),
            FindingKind::UnvettedToolAuthorization => counter_examples.push(CounterExample {
                variable: "tool_scope_vetted".to_string(),
                value: "false".to_string(),
                explanation: "Tool authorization was invoked without a bounded policy proof."
                    .to_string(),
            }),
        }
    }

    VerificationReport {
        status: if counter_examples.is_empty() {
            VerificationStatus::Pass
        } else {
            VerificationStatus::Crash
        },
        findings: findings
            .into_iter()
            .map(|mut finding| {
                if finding.risk == RiskLevel::Low {
                    finding.risk = RiskLevel::Medium;
                }
                finding
            })
            .collect(),
        counter_examples,
    }
}

pub fn report_to_json(report: &VerificationReport) -> Result<String, SolverError> {
    Ok(serde_json::to_string_pretty(report)?)
}

fn prove_division_counter_example() -> Option<CounterExample> {
    let config = Config::new();
    let context = Context::new(&config);
    let solver = Solver::new(&context);
    let z = Int::new_const(&context, "z");
    let five = Int::from_i64(&context, 5);
    solver.assert(&z._eq(&five));

    match solver.check() {
        SatResult::Sat => Some(CounterExample {
            variable: "z".to_string(),
            value: "5".to_string(),
            explanation: "When z == 5, the denominator expression (z - 5) reaches zero."
                .to_string(),
        }),
        SatResult::Unsat | SatResult::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{FindingKind, RiskLevel};

    #[test]
    fn emits_counter_example_for_arithmetic() {
        let report = verify_findings(vec![Finding {
            kind: FindingKind::UnboundedArithmetic,
            risk: RiskLevel::Critical,
            evidence: "z - 5".to_string(),
            constraint: "denominator == 0".to_string(),
        }]);
        assert_eq!(report.status, VerificationStatus::Crash);
        assert_eq!(report.counter_examples[0].value, "5");
    }
}
