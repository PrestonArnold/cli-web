mod runtime;
use runtime::WasmRuntime;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let mut rt = WasmRuntime::new("../wasm/target/wasm32-unknown-unknown/release/wasm.wasm")?;

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
        let output = rt.run(parts[0], &parts[1..])?;
        println!("{}", output);
    }

    Ok(())
}