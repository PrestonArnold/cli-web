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

fn read_input(ptr: i32, len: i32) -> String {
    let bytes = unsafe {
        slice::from_raw_parts(ptr as *const u8, len as usize)
    };
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
pub extern "C" fn handle_command(ptr: i32, len: i32) -> i32 {
    let input_str = read_input(ptr, len);
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let result = match input.command.as_str() {
        "help" => "commands: help, about, projects".to_string(),
        "about" => "Hi, I'm Preston Arnold.".to_string(),
        "projects" => "ssh-os, ai-tools".to_string(),
        _ => format!("unknown command: {}", input.command),
    };

    let output = Output { output: result };
    let json = serde_json::to_string(&output).unwrap();

    let bytes = json.into_bytes();
    let len = bytes.len();

    let ptr = alloc(len as i32);

    unsafe {
        let dest = slice::from_raw_parts_mut(ptr as *mut u8, len);
        dest.copy_from_slice(&bytes);
    }

    ptr
}