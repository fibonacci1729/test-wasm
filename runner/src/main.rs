use anyhow::{anyhow, Result, Context};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use std::{fs, env, path::Path, collections::BTreeSet};
use wasmtime::{
    component::{types, Component, Linker},
    Config, Engine, Store,
};

const TEST_PREFIX: &str = "test-";

struct TestRunner;

impl TestRunner {
    const ADAPTER_BYTES: &'static [u8] = include_bytes!("../../adapter/wasi_snapshot_preview1.reactor.wasm");

    fn collect_exports(engine: &Engine, component_ty: types::Component) -> Result<BTreeSet<String>> {
        component_ty
            .exports(engine)
            .try_fold(BTreeSet::new(), |mut exports, (name, _item)| {
                exports.insert(name.into());
                // TODO: ensure item is a func
                Ok(exports)
            })
    }

    fn read_component(path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let bytes = &fs::read(&path).context("failed to read input file")?;
        wit_component::ComponentEncoder::default()
            .adapter("wasi_snapshot_preview1", Self::ADAPTER_BYTES)?
            .module(&bytes)?
            .encode()
    }

    fn run(path: impl AsRef<Path>) -> Result<()> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;
        let binary = Self::read_component(path)?;
        let component = Component::from_binary(&engine, &binary)?;
        let table = Default::default();
        let ctx = WasiCtxBuilder::new().build();
        let data = Data { ctx, table };

        let mut store = Store::new(&engine, data);
        
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::command::sync::add_to_linker(&mut linker)?;

        let exports = Self::collect_exports(&engine, component.component_type())?;
        for exported in exports {
            if let Some(test_name) = exported.strip_prefix(TEST_PREFIX) {
                let test_instance = linker.instantiate(&mut store, &component)?;

                let test_func = test_instance
                    .exports(&mut store)
                    .root()
                    .func(&exported)
                    .unwrap();

                match test_func.call(&mut store, &[], &mut []) {
                    Ok(()) => {
                        println!("{test_name} ... OK!");
                    }
                    Err(err) => {
                        eprintln!("error: {test_name} test failed: {err}");
                    }
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    TestRunner::run(
        env::args()
            .skip(1)
            .next()
            .ok_or(anyhow!("missing argument"))?,
    )
}

struct Data {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for Data {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}