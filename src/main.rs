use deno_runtime::{
    deno_core::{anyhow::anyhow, error::AnyError, ModuleId, ModuleSpecifier},
    permissions::PermissionsContainer,
    worker::{MainWorker, WorkerOptions},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

struct Handlers {
    handlers: HashMap<String, Handler>,
}

impl Handlers {
    fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    async fn load(&mut self, id: String, path: PathBuf) -> Result<(), AnyError> {
        let handler = Handler::new(path).await?;
        self.handlers.insert(id, handler);
        Ok(())
    }

    fn unload(&mut self, id: &str) -> Result<bool, AnyError> {
        match self.handlers.remove(id) {
            Some(_handler) => Ok(true),
            None => Ok(false),
        }
    }

    async fn reload(&mut self, id: String, path: PathBuf) -> Result<(), AnyError> {
        self.unload(&id)?;
        self.load(id, path).await
    }

    async fn reload_id(&mut self, id: &str) -> Result<(), AnyError> {
        match self.handlers.get(id) {
            Some(handler) => {
                self.reload(id.into(), handler.path.clone()).await?;
                Ok(())
            }
            None => Err(AnyError::msg("Handler not found")),
        }
    }
}

struct Handler {
    path: PathBuf,
    worker: MainWorker,
    module_id: ModuleId,
}

impl Handler {
    async fn new(js_path: PathBuf) -> Result<Handler, AnyError> {
        let main_module = ModuleSpecifier::from_file_path(js_path.clone())
            .map_err(|_| anyhow!("main not found"))?;

        let mut worker = MainWorker::bootstrap_from_options(
            main_module.clone(),
            PermissionsContainer::allow_all(),
            WorkerOptions {
                ..Default::default()
            },
        );

        let module_id = worker.preload_main_module(&main_module).await?;
        worker.evaluate_module(module_id).await?;
        worker.run_event_loop(false).await?;

        Ok(Handler {
            path: js_path,
            worker,
            module_id,
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), AnyError> {
    // let platform = deno_runtime::deno_core::v8::new_default_platform(0, false).make_shared();
    // deno_runtime::deno_core::v8::V8::initialize_platform(platform);
    // deno_runtime::deno_core::v8::V8::initialize();
    // deno_runtime::deno_core::JsRuntime::init_platform(Some(platform));

    let mut handlers = Handlers::new();
    let handler1_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("handler1.js");

    handlers
        .load("handler1_0".into(), handler1_path.clone())
        .await?;

    handlers.load("handler1_1".into(), handler1_path).await?;

    let _ = handlers.reload_id("handler1_0").await?;
    let _ = handlers.reload_id("handler1_1").await?;

    Ok(())
}
