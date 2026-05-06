use crate::constructive::calldata::element_type::CalldataElementType;
use crate::executive::{
    executable::{
        compiler::compiler::ProgramCompiler,
        executable::Program,
        method::{
            compiler::compiler::MethodCompiler, method_type::MethodType, program_method::ProgramMethod,
        },
    },
    opcode::{compiler::compiler::OpcodeCompiler, opcode::Opcode},
};

/// Compiles program-related structures from CLI arguments.
pub fn comp_command(parts: Vec<&str>) {
    match parts.get(1).copied() {
        Some("programmethod") => comp_program_method(parts),
        Some("program") => comp_program(parts),
        _ => eprintln!("Usage: comp <programmethod|program> ..."),
    }
}

fn comp_program_method(parts: Vec<&str>) {
    let method_name = match parts.get(2) {
        Some(v) => (*v).to_string(),
        None => {
            eprintln!("Usage: comp programmethod <method_name> <callable|internal|readonly> <arg_types_csv> <0x_script_bytes>");
            return;
        }
    };

    let method_type = match parts.get(3).and_then(|v| parse_method_type(v)) {
        Some(v) => v,
        None => {
            eprintln!("Invalid method type. Use callable | internal | readonly.");
            return;
        }
    };

    let arg_types = match parts.get(4).and_then(|v| parse_arg_types(v)) {
        Some(v) => v,
        None => {
            eprintln!("Invalid calldata element types. Example: u8,bytes84,varbytes");
            return;
        }
    };

    let script_bytes = match parts.get(5).and_then(|v| parse_hex_bytes(v)) {
        Some(v) => v,
        None => {
            eprintln!("Invalid script bytes. Expected 0x-prefixed hex.");
            return;
        }
    };

    let script = match decode_script_opcodes(script_bytes) {
        Some(v) => v,
        None => {
            eprintln!("Invalid script bytes: unable to decode opcodes.");
            return;
        }
    };

    let method = match ProgramMethod::new(method_name, method_type, arg_types, script) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to construct ProgramMethod: {:?}", err);
            return;
        }
    };

    let compiled = match method.compile() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to compile ProgramMethod: {}", err);
            return;
        }
    };

    println!("0x{}", hex::encode(compiled));
}

fn comp_program(parts: Vec<&str>) {
    let Some((program_name, metadata, method_count, method_bytes_args)) =
        parse_comp_program_arguments(&parts)
    else {
        eprintln!("Usage: comp program <program_name> <metadata_hex_or_0x> <num_methods> <0x_method1> ... <0x_methodN>");
        eprintln!("Example: comp program my awesome app 0x 1 0x0568656c6c6f00010005005151515151");
        return;
    };

    let mut methods = Vec::<ProgramMethod>::with_capacity(method_count);
    for (idx, method_hex) in method_bytes_args.iter().enumerate() {
        let mut stream = match parse_hex_bytes(method_hex) {
            Some(v) => v.into_iter(),
            None => {
                eprintln!("Invalid method bytes at index {}.", idx);
                return;
            }
        };

        let method = match ProgramMethod::decompile(&mut stream) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Unable to decode method {}: {}", idx, err);
                return;
            }
        };
        methods.push(method);
    }

    let program = match Program::new(program_name, metadata, methods) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to construct Program: {}", err);
            return;
        }
    };

    let compiled = match program.compile() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to compile Program: {}", err);
            return;
        }
    };

    println!("0x{}", hex::encode(compiled));
}

fn parse_comp_program_arguments<'a>(
    parts: &'a Vec<&'a str>,
) -> Option<(String, Option<Vec<u8>>, usize, &'a [&'a str])> {
    // Expected minimum:
    // comp program <program_name> <metadata> <num_methods>
    if parts.len() < 5 {
        return None;
    }

    // We parse from right to support multi-word program names:
    // ... <metadata> <num_methods> <method1> ... <methodN>
    for num_idx in 4..parts.len() {
        let method_count = match parts[num_idx].parse::<usize>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method_bytes_args = &parts[(num_idx + 1)..];
        if method_bytes_args.len() != method_count {
            continue;
        }

        let metadata_idx = num_idx.checked_sub(1)?;
        if metadata_idx < 3 {
            continue;
        }

        let metadata = match parse_hex_bytes(parts[metadata_idx]) {
            Some(v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            }
            None => continue,
        };

        let program_name = parts[2..metadata_idx].join(" ");
        if program_name.trim().is_empty() {
            continue;
        }

        // Validate all method hex blobs up front for better argument disambiguation.
        if method_bytes_args.iter().any(|m| parse_hex_bytes(m).is_none()) {
            continue;
        }

        return Some((program_name, metadata, method_count, method_bytes_args));
    }

    None
}

fn parse_method_type(v: &str) -> Option<MethodType> {
    match v.trim().to_ascii_lowercase().as_str() {
        "callable" => Some(MethodType::Callable),
        "internal" => Some(MethodType::Internal),
        "readonly" => Some(MethodType::ReadOnly),
        _ => None,
    }
}

fn parse_arg_types(v: &str) -> Option<Vec<CalldataElementType>> {
    let normalized = v.trim();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }

    let mut out = Vec::new();
    for raw in normalized.split(',') {
        let token = raw.trim().to_ascii_lowercase();
        let parsed = if token == "u8" {
            Some(CalldataElementType::U8)
        } else if token == "u16" {
            Some(CalldataElementType::U16)
        } else if token == "u32" {
            Some(CalldataElementType::U32)
        } else if token == "u64" {
            Some(CalldataElementType::U64)
        } else if token == "bool" {
            Some(CalldataElementType::Bool)
        } else if token == "account" {
            Some(CalldataElementType::Account)
        } else if token == "contract" {
            Some(CalldataElementType::Contract)
        } else if token == "varbytes" {
            Some(CalldataElementType::Varbytes)
        } else if token == "payable" {
            Some(CalldataElementType::Payable)
        } else if let Some(rest) = token.strip_prefix("bytes") {
            let len = rest.parse::<u16>().ok()?;
            if !(1..=256).contains(&len) {
                return None;
            }
            Some(CalldataElementType::Bytes((len - 1) as u8))
        } else {
            None
        }?;

        out.push(parsed);
    }

    Some(out)
}

fn decode_script_opcodes(script_bytes: Vec<u8>) -> Option<Vec<Opcode>> {
    let mut stream = script_bytes.into_iter();
    let mut out = Vec::<Opcode>::new();

    while stream.len() > 0 {
        let opcode = Opcode::decompile(&mut stream).ok()?;
        out.push(opcode);
    }

    Some(out)
}

fn parse_hex_bytes(s: &str) -> Option<Vec<u8>> {
    hex::decode(s.trim_start_matches("0x")).ok()
}
