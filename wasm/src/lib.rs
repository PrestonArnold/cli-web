use serde::{Deserialize, Serialize};
use std::{mem, slice};

#[derive(Deserialize)]
pub struct Input {
    command: String,
    args: Vec<String>,
}

#[derive(Serialize)]
struct Output {
    output: String,
}

trait Command {
    fn name(&self) -> &'static str;
    fn run(&self, args: &[String]) -> String;
}

struct Help;
impl Command for Help {
    fn name(&self) -> &'static str { "help" }
    fn run(&self, _args: &[String]) -> String {
        let names: Vec<&str> = registry().iter().map(|c| c.name()).collect();
        format!("commands: {}", names.join(", "))
    }
}

struct About;
impl Command for About {
    fn name(&self) -> &'static str { "about" }
    
    fn run(&self, args: &[String]) -> String {
        "Hi, I'm Preston Arnold.".to_string()
    }
}

struct Projects;
impl Command for Projects {
    fn name(&self) -> &'static str { "projects" }
    
    fn run(&self, _args: &[String]) -> String {
        "ssh-os, ai-tools".to_string()
    }
}

fn registry() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(Help),
        Box::new(About),
        Box::new(Projects),
    ]
}

fn read_input(ptr: i32, len: i32) -> String {
    let bytes = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: i32) -> i32 {
    let mut buf = Vec::<u8>::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_command(ptr: i32, len: i32) -> i64 {
    let input_str = read_input(ptr, len);
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let commands = registry();
    let result = commands
        .iter()
        .find(|c| c.name() == input.command.as_str())
        .map(|c| c.run(&input.args))
        .unwrap_or_else(|| format!("unknown command: {}", input.command));

    let output = Output { output: result };
    let json = serde_json::to_string(&output).unwrap();
    let bytes = json.into_bytes();
    let len = bytes.len();
    let ptr = alloc(len as i32);

    unsafe {
        let dest = slice::from_raw_parts_mut(ptr as *mut u8, len);
        dest.copy_from_slice(&bytes);
    }

    ((ptr as i64) << 32) | (len as i64)
}