use crate::executive::{
    executable::{
        compiler::compiler::ExecutableCompiler,
        executable::Executable,
        method::{compiler::compiler::MethodCompiler, method::ExecutableMethod},
    },
    opcode::{compiler::compiler::OpcodeCompiler, opcode::Opcode},
};
use serde_json::to_string_pretty;

/// Prints the current set of lifts in the wallet.
pub fn decomp_command(parts: Vec<&str>) {
    match parts.get(1) {
        Some(part) => match part.to_owned() {
            "executable" => decomp_executable(parts),
            "method" => decomp_method(parts),
            "script" => decomp_script(parts),
            _ => eprintln!("Unknown command."),
        },
        None => eprintln!("Incorrect usage."),
    }
}

/// Decompiles an executable from bytes.
fn decomp_executable(parts: Vec<&str>) {
    let executable_bytes_str = match parts.get(2) {
        Some(executable_bytes_str) => executable_bytes_str,
        None => {
            eprintln!("Incorrect usage.");
            return;
        }
    };

    let mut executable_bytestream = match hex::decode(executable_bytes_str) {
        Ok(executable_bytes) => executable_bytes.into_iter(),
        Err(_) => {
            eprintln!("Invalid executable bytes.");
            return;
        }
    };

    let executable = match Executable::decompile(&mut executable_bytestream) {
        Ok(executable) => executable,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let pretty_json = to_string_pretty(&executable.json()).unwrap();

    println!("{}", pretty_json);
}

/// Decompiles a method from bytes.
fn decomp_method(parts: Vec<&str>) {
    let method_bytes_str = match parts.get(2) {
        Some(method_bytes_str) => method_bytes_str,
        None => {
            eprintln!("Incorrect usage.");
            return;
        }
    };

    let mut method_bytestream = match hex::decode(method_bytes_str) {
        Ok(method_bytes) => method_bytes.into_iter(),
        Err(_) => {
            eprintln!("Invalid method bytes.");
            return;
        }
    };

    let method = match ExecutableMethod::decompile(&mut method_bytestream) {
        Ok(method) => method,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let pretty_json = to_string_pretty(&method.json()).unwrap();

    println!("{}", pretty_json);
}

fn decomp_script(parts: Vec<&str>) {
    let script_bytes_str = match parts.get(2) {
        Some(script_bytes_str) => script_bytes_str,
        None => {
            eprintln!("Incorrect usage.");
            return;
        }
    };

    let mut opcode_bytestream = match hex::decode(script_bytes_str) {
        Ok(opcode_bytes) => opcode_bytes.into_iter(),
        Err(_) => {
            eprintln!("Invalid opcode bytes.");
            return;
        }
    };

    let mut opcodes = Vec::<Opcode>::new();

    loop {
        let opcode = match Opcode::decompile(&mut opcode_bytestream) {
            Ok(opcode) => opcode,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        opcodes.push(opcode);

        if opcode_bytestream.len() == 0 {
            break;
        }
    }

    let opcodes: Vec<String> = opcodes.iter().map(|opcode| opcode.to_string()).collect();

    println!("{}", opcodes.join(" "));
}
