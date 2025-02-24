use std::io::Read;
use std::io::Seek;
use std::path::Path;

use crate::error::SandboxError;
use anyhow::bail;
use tokio::time::{interval, Duration};
use tracing::instrument;
use tracing::Instrument;
use wasmtime::Config;
use wasmtime::Trap;
use wasmtime::{Engine, Linker, Module, ResourceLimiter, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::preview1::WasiP1Ctx;

const WASM_MINIMUM_MEMORY_SIZE: u64 = bytesize::KIB * 64 * 17;
const WASM_INSTANCE_MEMORY_LIMIT: u64 = WASM_MINIMUM_MEMORY_SIZE + bytesize::MB * 100;
const TICKS_BEFORE_TIMEOUT: u64 = 5;

struct WasmStoreData {
    wasi: WasiP1Ctx,
    memory_limiter: MemoryLimiter,
}

struct MemoryLimiter {
    limiter: StoreLimits,
    memory_limit_exceeded: bool,
}

impl MemoryLimiter {
    fn new(limiter: StoreLimits) -> Self {
        Self {
            limiter,
            memory_limit_exceeded: false,
        }
    }

    fn memory_limit_exceeded(&self) -> bool {
        self.memory_limit_exceeded
    }
}

impl ResourceLimiter for MemoryLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        let is_rejected = self.limiter.memory_growing(_current, desired, _maximum);
        if let Ok(is_rejected) = is_rejected {
            if is_rejected {
                self.memory_limit_exceeded = true;
            }
        }
        is_rejected
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        self.limiter.table_growing(current, desired, maximum)
    }

    fn table_grow_failed(&mut self, _error: anyhow::Error) -> anyhow::Result<()> {
        Err(anyhow::format_err!("table_grow_failed"))
    }

    fn memory_grow_failed(&mut self, _error: anyhow::Error) -> anyhow::Result<()> {
        self.memory_limit_exceeded = true;
        Err(anyhow::format_err!("memory_grow_failed"))
    }
}

fn is_deadline_error(err: &Trap) -> bool {
    err.to_string().starts_with("epoch deadline")
}

fn run_wasm_instance(
    module: &Module,
    engine: &Engine,
    mut store: &mut Store<WasmStoreData>,
) -> anyhow::Result<Option<anyhow::Error>> {
    use wasmtime_wasi::preview1;

    let mut linker: Linker<WasmStoreData> = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |t| &mut t.wasi)?;
    let pre = linker.instantiate_pre(&module)?;
    let instance = pre.instantiate(&mut store)?;

    let err = instance
        .get_typed_func::<(), ()>(&mut store, "_start")?
        .call(&mut store, ())
        .err();

    Ok(err)
}

fn run_wasm_timeout(engine: Engine) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_millis(1000));
        for _ in 0..TICKS_BEFORE_TIMEOUT {
            interval.tick().await;
            engine.increment_epoch();
        }
    })
}

use std::fs::File;

#[cfg(target_os = "linux")]
fn create_stdout_file() -> std::io::Result<File> {
    use tempfile::tempfile;
    tempfile()
}

#[cfg(target_os = "macos")]
fn create_stdout_file() -> std::io::Result<File> {
    use tempfile::tempfile;
    tempfile()
}

fn execute_wasm_instance(module: Module, engine: Engine) -> anyhow::Result<String> {
    use wasmtime_wasi::WasiCtxBuilder;

    let mut memfile = create_stdout_file()?;
    let stdout_file = wasmtime_wasi::OutputFile::new(memfile.try_clone()?);
    let wasi = WasiCtxBuilder::new().stdout(stdout_file).build_p1();

    let mut store = create_wasm_store(&engine, wasi);
    store.set_epoch_deadline(TICKS_BEFORE_TIMEOUT);

    let timeout = run_wasm_timeout(engine.clone());
    let err = run_wasm_instance(&module, &engine, &mut store)?;

    if let Some(err) = err {
        if is_deadline_error(&err.downcast::<Trap>()?) {
            tracing::info!("SandboxError::Timeout");
            bail!(SandboxError::Timeout)
        }

        if store.data().memory_limiter.memory_limit_exceeded() {
            timeout.abort();
            tracing::info!("SandboxError::OOM");
            bail!(SandboxError::OOM)
        }
    }

    timeout.abort();
    if let Err(e) = memfile.rewind() {
        bail!("failed to rewind: {e}")
    }

    let mut output = String::new();
    memfile.read_to_string(&mut output)?;

    Ok(output)
}

fn create_wasm_store(engine: &Engine, wasi: WasiP1Ctx) -> Store<WasmStoreData> {
    let store_limits = StoreLimitsBuilder::new()
        .memory_size(WASM_INSTANCE_MEMORY_LIMIT as usize)
        .build();
    let memory_limiter = MemoryLimiter::new(store_limits);
    let mut store = Store::new(
        engine,
        WasmStoreData {
            wasi,
            memory_limiter,
        },
    );
    store.limiter(|data| &mut data.memory_limiter);
    store
}

#[instrument(skip_all, name = "Executing WASM instance", fields(
    service.name = "typerust"
))]
pub async fn execute_wasm(engine: Engine, module_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let module = Module::from_file(&engine, module_path)?;
    let task_span = tracing::info_span!("Running execution task");
    tokio::task::spawn_blocking(move || execute_wasm_instance(module, engine))
        .instrument(task_span)
        .await?
}

pub fn create_interruptable_engine() -> Engine {
    let mut engine_config = Config::new();
    engine_config.epoch_interruption(true);
    Engine::new(&engine_config).expect("failed to initialize wasm engine")
}
