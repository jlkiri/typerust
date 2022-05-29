use crate::error::Result;
use crate::wasm::execute_wasm;
use crate::State;
use anyhow::bail;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;
use tokio::time::{Duration, Instant};
use tracing::instrument;

const CRATE_NAME: &str = "playground";

struct Compiler {
    #[allow(dead_code)]
    tempdir: TempDir,
    output_dir: PathBuf,
    input_file: PathBuf,
}

macro_rules! add_ext {
    ($filename:expr, $ext:expr) => {
        format!("{}.{}", $filename, $ext)
    };
}

type OutputText = String;
type ExecutablePath = PathBuf;

#[derive(Debug)]
enum BuildResult {
    Success {
        elapsed: Duration,
        executable: ExecutablePath,
    },
    Failure(OutputText),
}

fn bytes_to_string(vec: Vec<u8>) -> anyhow::Result<String> {
    let res = String::from_utf8(vec)?;
    Ok(res)
}

impl Compiler {
    async fn new() -> anyhow::Result<Self> {
        let tempdir = tempfile::Builder::new()
            .prefix("playground-")
            .rand_bytes(10)
            .tempdir();

        if tempdir.is_err() {
            bail!("failed to create tempdir")
        }

        let tempdir = tempdir.unwrap();

        let filename = add_ext!(CRATE_NAME, "rs");
        let input_file = tempdir.path().join(filename);
        let output_dir = tempdir.path().join("out");

        let create_dir_result = fs::create_dir(&output_dir).await;
        if create_dir_result.is_err() {
            bail!("failed to create output dir")
        }

        Ok(Self {
            tempdir,
            input_file,
            output_dir,
        })
    }

    async fn write_source_code(&self, code: String) -> anyhow::Result<()> {
        let write_result = fs::write(&self.input_file, code).await;
        if write_result.is_err() {
            bail!("failed to write code to tempfile")
        }
        Ok(())
    }

    async fn compile(&self, code: String) -> anyhow::Result<BuildResult> {
        self.write_source_code(code).await?;

        let start = Instant::now();
        let mut cmd = Command::new("rustc");
        let cmd = cmd
            .arg("--out-dir")
            .arg(&self.output_dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&self.input_file);
        let output = cmd.output().await;

        if output.is_err() {
            bail!("failed to execute child command");
        }

        let output = output.unwrap();
        let elapsed = start.elapsed();

        if !output.status.success() {
            let string = bytes_to_string(output.stderr);
            if string.is_err() {
                bail!("failed to parse command output as utf-8");
            }
            let string = string.unwrap();
            return Ok(BuildResult::Failure(string));
        }

        let filename = add_ext!(CRATE_NAME, "wasm");
        let executable = self.output_dir.join(filename);

        Ok(BuildResult::Success {
            executable,
            elapsed,
        })
    }
}

#[derive(Serialize)]
pub struct Success {
    elapsed: f32,
    output: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum HandlerResponse {
    Success(Success),
    Error(String),
}

#[instrument(skip_all, name = "Run playground code", fields(
    service.name = "typerust"
))]
pub async fn run(code: String, state: Arc<State>) -> Result<HandlerResponse> {
    let compiler = Compiler::new().await?;
    let result = compiler.compile(code).await?;
    match result {
        BuildResult::Success {
            elapsed,
            executable,
        } => {
            tracing::info!("successfully compiled playground code");
            let elapsed = elapsed.as_secs_f32();
            let output = execute_wasm(state.engine.clone(), executable).await?;
            let success = Success {
                elapsed,
                output: Some(output),
            };
            Ok(HandlerResponse::Success(success))
        }
        BuildResult::Failure(output) => {
            tracing::error!("failed to build playground code");
            Ok(HandlerResponse::Error(output))
        }
    }
}

#[instrument(skip_all, name = "Build playground code", fields(
    service.name = "typerust"
))]
pub async fn build(code: String) -> Result<HandlerResponse> {
    let sandbox = Compiler::new().await?;
    let build_result = sandbox.compile(code).await?;
    match build_result {
        BuildResult::Success { elapsed, .. } => {
            tracing::info!("successfully compiled playground code");
            let elapsed = elapsed.as_secs_f32();
            let success = Success {
                elapsed,
                output: None,
            };
            Ok(HandlerResponse::Success(success))
        }
        BuildResult::Failure(output) => {
            tracing::error!("failed to build playground code");
            Ok(HandlerResponse::Error(output))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_interruptable_engine, error::SandboxError};
    use once_cell::sync::Lazy;

    static STATE: Lazy<Arc<State>> = Lazy::new(|| {
        Arc::new(State {
            engine: create_interruptable_engine(),
        })
    });

    #[tokio::test]
    async fn kill_on_oom() -> anyhow::Result<()> {
        let sandbox = Compiler::new().await?;
        let code = r#"
fn main() {
    let mut nums: Vec<Box<u64>> = vec![];
    for i in 0..100000000 {
        nums.push(Box::new(i));
    }
}
        "#;

        let result = sandbox.compile(code.into()).await?;

        assert!(matches!(result, BuildResult::Success { .. }));
        if let BuildResult::Success { executable, .. } = result {
            let result = execute_wasm(STATE.engine.clone(), executable).await.err();
            let error = result.map(|err| err.downcast::<SandboxError>());
            assert!(matches!(error, Some(Ok(SandboxError::OOM))));
        }

        Ok(())
    }

    #[tokio::test]
    async fn stay_within_memory_limit() -> anyhow::Result<()> {
        let sandbox = Compiler::new().await?;
        let code = r#"
fn main() {
    let mut nums: Vec<Box<u64>> = vec![];
    for i in 0..10 {
        nums.push(Box::new(i));
    }
}
        "#;

        let result = sandbox.compile(code.into()).await?;

        assert!(matches!(result, BuildResult::Success { .. }));

        if let BuildResult::Success { executable, .. } = result {
            let result = execute_wasm(STATE.engine.clone(), executable).await.err();
            assert!(matches!(result, None));
        }

        Ok(())
    }

    #[tokio::test]
    async fn timeout_on_loop() -> anyhow::Result<()> {
        let sandbox = Compiler::new().await?;
        let code = r#"
fn main() {
    loop {}
}
        "#;

        let result = sandbox.compile(code.into()).await?;

        assert!(matches!(result, BuildResult::Success { .. }));

        if let BuildResult::Success { executable, .. } = result {
            let result = execute_wasm(STATE.engine.clone(), executable).await.err();
            let error = result.map(|err| err.downcast::<SandboxError>());
            assert!(matches!(error, Some(Ok(SandboxError::Timeout))));
        }

        Ok(())
    }
}
