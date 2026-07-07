use thiserror::Error;
use tokio::task;
use zevq_backend::{audit::audit_payload_json, solver::SolverError};

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
    let report_json = task::spawn_blocking(move || audit_payload_json(&payload)).await??;
    println!("{report_json}");
    Ok(())
}
