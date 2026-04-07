use wasmtime::*;
use serde_json::json;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();

    let module = Module::from_file(
        &engine,
        "../wasm/target/wasm32-unknown-unknown/release/wasm.wasm"
    )?;

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("no memory found");

    let alloc = instance
        .get_typed_func::<i32, i32>(&mut store, "alloc")?;

    let handle = instance
        .get_typed_func::<(i32, i32), i64>(&mut store, "handle_command")?;

    println!("Welcome to prestonarnold.uk\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        let json_input = json!({
            "command": command,
            "args": args
        })
        .to_string();

        let bytes = json_input.as_bytes();
        let len = bytes.len() as i32;

        let ptr = alloc.call(&mut store, len)?;

        memory.write(&mut store, ptr as usize, bytes)?;

        let result = handle.call(&mut store, (ptr, len))?;

        let result_ptr = (result >> 32) as i32;
        let result_len = (result & 0xffffffff) as i32;

        let mut buffer = vec![0u8; result_len as usize];
        memory.read(&mut store, result_ptr as usize, &mut buffer)?;

        let output_str = String::from_utf8(buffer)?;
        let parsed: serde_json::Value = serde_json::from_str(&output_str)?;

        println!("{}", parsed["output"]);
    }

    Ok(())
}