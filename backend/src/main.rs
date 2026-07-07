use thiserror::Error;
use tokio::task;
use zevq_backend::{
    parser::parse_rust_source,
    solver::{report_to_json, verify_findings, SolverError},
};

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("verification task failed: {0}")]
    Join(#[from] task::JoinError),
    #[error(transparent)]
    Solver(#[from] SolverError),
}

#[tokio::main]
async fn main() -> Result<(), PipelineError> {
    let payload = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fn main(){ let x = 1 / (z - 5); }".to_string());
    let report = task::spawn_blocking(move || verify_findings(parse_rust_source(&payload))).await?;
    println!("{}", report_to_json(&report)?);
    Ok(())
}
