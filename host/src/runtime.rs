use wasmtime::*;
use serde_json::json;
use anyhow::Result;

pub struct WasmRuntime {
    store: Store<()>,
    memory: Memory,
    alloc: TypedFunc<i32, i32>,
    handle: TypedFunc<(i32, i32), i64>,
}

impl WasmRuntime {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("no memory found");

        let alloc = instance.get_typed_func::<i32, i32>(&mut store, "alloc")?;
        let handle = instance.get_typed_func::<(i32, i32), i64>(&mut store, "handle_command")?;

        Ok(Self { store, memory, alloc, handle })
    }

    pub fn run(&mut self, command: &str, args: &[&str]) -> Result<String> {
        let json_input = json!({ "command": command, "args": args }).to_string();
        let bytes = json_input.as_bytes();
        let len = bytes.len() as i32;

        let ptr = self.alloc.call(&mut self.store, len)?;
        self.memory.write(&mut self.store, ptr as usize, bytes)?;

        let result = self.handle.call(&mut self.store, (ptr, len))?;
        let result_ptr = (result >> 32) as i32;
        let result_len = (result & 0xffffffff) as i32;

        let mut buffer = vec![0u8; result_len as usize];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;

        let output_str = String::from_utf8(buffer)?;
        let parsed: serde_json::Value = serde_json::from_str(&output_str)?;

        Ok(parsed["output"].as_str().unwrap_or("").to_string())
    }
}