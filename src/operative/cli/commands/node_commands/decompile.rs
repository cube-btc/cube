use crate::executive::{
    executable::{
        compiler::compiler::ProgramCompiler,
        executable::Executable,
        method::{compiler::compiler::MethodCompiler, program_method::ProgramMethod},
    },
    opcode::{compiler::compiler::OpcodeCompiler, opcode::Opcode},
};
use serde_json::to_string_pretty;

fn parse_hex_bytes(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex_str.trim().trim_start_matches("0x"))
}

/// Prints the current set of lifts in the wallet.
pub fn decompile_command(parts: Vec<&str>) {
    match parts.get(1) {
        Some(part) => match part.to_owned() {
            "program" | "executable" => decomp_program(parts),
            "programmethod" | "method" => decomp_programmethod(parts),
            "script" => decomp_script(parts),
            _ => eprintln!("Unknown command."),
        },
        None => eprintln!("Incorrect usage."),
    }
}

/// Decompiles a program from bytes.
fn decomp_program(parts: Vec<&str>) {
    let program_bytes_str = match parts.get(2) {
        Some(program_bytes_str) => program_bytes_str,
        None => {
            eprintln!("Incorrect usage.");
            return;
        }
    };

    let mut program_bytestream = match parse_hex_bytes(program_bytes_str) {
        Ok(program_bytes) => program_bytes.into_iter(),
        Err(_) => {
            eprintln!("Invalid program bytes.");
            return;
        }
    };

    let program = match Executable::decompile(&mut program_bytestream) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let pretty_json = to_string_pretty(&program.json()).unwrap();

    println!("{}", pretty_json);
}

/// Decompiles a program method from bytes.
fn decomp_programmethod(parts: Vec<&str>) {
    let method_bytes_str = match parts.get(2) {
        Some(method_bytes_str) => method_bytes_str,
        None => {
            eprintln!("Incorrect usage.");
            return;
        }
    };

    let mut method_bytestream = match parse_hex_bytes(method_bytes_str) {
        Ok(method_bytes) => method_bytes.into_iter(),
        Err(_) => {
            eprintln!("Invalid method bytes.");
            return;
        }
    };

    let method = match ProgramMethod::decompile(&mut method_bytestream) {
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

    let mut opcode_bytestream = match parse_hex_bytes(script_bytes_str) {
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
