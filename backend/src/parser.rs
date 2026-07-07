use serde::{Deserialize, Serialize};
use syn::{
    visit::Visit, BinOp, Expr, ExprBinary, ExprCall, ExprLit, ExprMethodCall, ExprPath, Lit,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingKind {
    UnboundedArithmetic,
    SqlStringConcat,
    UnvettedToolAuthorization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub kind: FindingKind,
    pub risk: RiskLevel,
    pub evidence: String,
    pub constraint: String,
}

#[derive(Default)]
struct SafetyVisitor {
    findings: Vec<Finding>,
}

pub fn parse_rust_source(source: &str) -> Vec<Finding> {
    let Ok(file) = syn::parse_file(source) else {
        return vec![Finding {
            kind: FindingKind::UnboundedArithmetic,
            risk: RiskLevel::High,
            evidence: "source failed to parse as Rust".to_string(),
            constraint: "parse_error == true".to_string(),
        }];
    };

    let mut visitor = SafetyVisitor::default();
    visitor.visit_file(&file);
    visitor.findings
}

pub fn parse_sql_string(query: &str) -> Vec<Finding> {
    let lowered = query.to_ascii_lowercase();
    let mut findings = Vec::new();

    if lowered.contains("+") || lowered.contains("format!") || lowered.contains("${") {
        findings.push(Finding {
            kind: FindingKind::SqlStringConcat,
            risk: RiskLevel::Critical,
            evidence: query.to_string(),
            constraint: "sql_parameterized == false".to_string(),
        });
    }

    if lowered.contains("tool") && (lowered.contains("admin") || lowered.contains("*")) {
        findings.push(Finding {
            kind: FindingKind::UnvettedToolAuthorization,
            risk: RiskLevel::High,
            evidence: query.to_string(),
            constraint: "tool_scope_vetted == false".to_string(),
        });
    }

    findings
}

impl<'ast> Visit<'ast> for SafetyVisitor {
    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        match &node.op {
            BinOp::Div(_) => self.findings.push(Finding {
                kind: FindingKind::UnboundedArithmetic,
                risk: RiskLevel::Critical,
                evidence: expr_to_hint(&node.right),
                constraint: "denominator == 0".to_string(),
            }),
            BinOp::Add(_)
                if expression_mentions_sql(&node.left) || expression_mentions_sql(&node.right) =>
            {
                self.findings.push(Finding {
                    kind: FindingKind::SqlStringConcat,
                    risk: RiskLevel::Critical,
                    evidence: format!(
                        "{} + {}",
                        expr_to_hint(&node.left),
                        expr_to_hint(&node.right)
                    ),
                    constraint: "sql_parameterized == false".to_string(),
                });
            }
            _ => {}
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        let function = expr_to_hint(&node.func);
        if function.contains("authorize") || function.contains("tool") {
            self.findings.push(Finding {
                kind: FindingKind::UnvettedToolAuthorization,
                risk: RiskLevel::High,
                evidence: function,
                constraint: "tool_scope_vetted == false".to_string(),
            });
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();
        if method == "push_str" || method == "replace" {
            let receiver = expr_to_hint(&node.receiver);
            if receiver.to_ascii_lowercase().contains("sql")
                || receiver.to_ascii_lowercase().contains("query")
            {
                self.findings.push(Finding {
                    kind: FindingKind::SqlStringConcat,
                    risk: RiskLevel::Critical,
                    evidence: format!("{receiver}.{method}"),
                    constraint: "sql_parameterized == false".to_string(),
                });
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn expression_mentions_sql(expr: &Expr) -> bool {
    expr_to_hint(expr).to_ascii_lowercase().contains("sql")
        || expr_to_hint(expr).to_ascii_lowercase().contains("query")
}

fn expr_to_hint(expr: &Expr) -> String {
    match expr {
        Expr::Path(ExprPath { path, .. }) => path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => value.value(),
        Expr::Binary(binary) => format!(
            "({} ? {})",
            expr_to_hint(&binary.left),
            expr_to_hint(&binary.right)
        ),
        _ => "expression".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_division_and_tool_auth() {
        let findings =
            parse_rust_source("fn main(){ let x = 1 / (z - 5); authorize_tool(admin); }");
        assert!(findings
            .iter()
            .any(|finding| finding.kind == FindingKind::UnboundedArithmetic));
        assert!(findings
            .iter()
            .any(|finding| finding.kind == FindingKind::UnvettedToolAuthorization));
    }
}
