mod runtime;
mod ssh;
use runtime::WasmRuntime;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wasm_path = "../wasm/target/wasm32-unknown-unknown/release/wasm.wasm";

    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("--ssh") {
        let rt = WasmRuntime::new(wasm_path)?;
        println!("SSH server listening on :2222");
        return ssh::serve(rt).await;
    }

    let mut rt = WasmRuntime::new(wasm_path)?;
    println!("Welcome to prestonarnold.uk\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() { continue; }
        if input == "exit" { break; }

        let parts: Vec<&str> = input.split_whitespace().collect();
        println!("{}", rt.run(parts[0], &parts[1..])?);
    }

    Ok(())
}