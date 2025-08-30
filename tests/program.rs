#[cfg(test)]
mod program_and_method_tests {
    use cube::{
        constructive::calldata::element_type::CallElementType,
        executive::{
            opcode::{
                opcode::Opcode,
                opcodes::{
                    flow::{op_returnall::OP_RETURNALL, op_returnerr::OP_RETURNERR},
                    push::{
                        op_2::OP_2, op_false::OP_FALSE, op_pushdata::OP_PUSHDATA, op_true::OP_TRUE,
                    },
                },
            },
            program::{
                compiler::compiler::ProgramCompiler,
                method::{
                    compiler::compiler::MethodCompiler, method::ProgramMethod,
                    method_type::MethodType,
                },
                program::Program,
            },
        },
    };

    #[test]
    fn method_construction_test() -> Result<(), String> {
        // Invalid name length.
        {
            let method_name = "a".to_string();
            let method_type = MethodType::Callable;
            let call_element_types = vec![CallElementType::U32, CallElementType::Account];
            let script = vec![
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_2(OP_2),
                Opcode::OP_PUSHDATA(OP_PUSHDATA(vec![0xde, 0xad, 0xbe, 0xef])),
                Opcode::OP_RETURNERR(OP_RETURNERR),
            ];

            let method = ProgramMethod::new(method_name, method_type, call_element_types, script);

            assert!(method.is_err());
        }

        // Invalid call element type count.
        {
            let method_name = "test_method".to_string();
            let method_type = MethodType::Callable;
            // Push more than the maximum (16) allowed call element types; 17
            let call_element_types = vec![
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
                CallElementType::U32,
            ];
            let script = vec![
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_2(OP_2),
                Opcode::OP_PUSHDATA(OP_PUSHDATA(vec![0xde, 0xad, 0xbe, 0xef])),
                Opcode::OP_RETURNERR(OP_RETURNERR),
            ];

            let method = ProgramMethod::new(method_name, method_type, call_element_types, script);

            assert!(method.is_err());
        }

        // Invalid opcode count.
        {
            let method_name = "test_method".to_string();
            let method_type = MethodType::Callable;
            let call_element_types = vec![CallElementType::U32, CallElementType::Account];
            // Push less than the minimum (4) allowed opcodes; 3
            let script = vec![
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_2(OP_2),
                Opcode::OP_PUSHDATA(OP_PUSHDATA(vec![0xde, 0xad, 0xbe, 0xef])),
            ];

            let method = ProgramMethod::new(method_name, method_type, call_element_types, script);

            assert!(method.is_err());
        }

        // Invalid script with non minimal push data.
        {
            let method_name = "test_method".to_string();
            let method_type = MethodType::Callable;
            let call_element_types = vec![CallElementType::U32, CallElementType::Account];
            // Push a non minimal push data.
            let script = vec![
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_2(OP_2),
                Opcode::OP_PUSHDATA(OP_PUSHDATA(vec![0x05])), // Should have been OP_5.
                Opcode::OP_RETURNALL(OP_RETURNALL),
            ];

            let method = ProgramMethod::new(method_name, method_type, call_element_types, script);

            assert!(method.is_err());
        }

        // Valid construction.
        {
            let method_name = "test_method".to_string();
            let method_type = MethodType::Callable;
            let call_element_types = vec![CallElementType::U32, CallElementType::Account];
            let script = vec![
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_2(OP_2),
                Opcode::OP_PUSHDATA(OP_PUSHDATA(vec![0xde, 0xad, 0xbe, 0xef])),
                Opcode::OP_RETURNALL(OP_RETURNALL),
            ];

            let method = ProgramMethod::new(method_name, method_type, call_element_types, script);

            assert!(method.is_ok());
        }
        Ok(())
    }

    #[test]
    fn method_compiler_test() -> Result<(), String> {
        let method_name = "test_method".to_string();
        let method_type = MethodType::Callable;
        let call_element_types = vec![CallElementType::U32, CallElementType::Account];
        let script = vec![
            Opcode::OP_TRUE(OP_TRUE),
            Opcode::OP_2(OP_2),
            Opcode::OP_PUSHDATA(OP_PUSHDATA(vec![0xde, 0xad, 0xbe, 0xef])),
            Opcode::OP_RETURNERR(OP_RETURNERR),
        ];

        let method =
            ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap();

        let mut method_compiled_bytestream = method.compile().unwrap().into_iter();

        let method_decompiled = ProgramMethod::decompile(&mut method_compiled_bytestream).unwrap();

        assert_eq!(method, method_decompiled);

        Ok(())
    }

    #[test]
    fn program_construction_test() -> Result<(), String> {
        // A program with no methods should fail.
        {
            let program_name = "test_program".to_string();
            let methods = vec![];

            let deployed_by = [0x00; 32];
            let program = Program::new(program_name, deployed_by, methods);

            assert!(program.is_err());
        }

        // A program with invalid name length should fail.
        {
            let program_name = "a".to_string();
            let method = {
                let method_name = "test_method".to_string();
                let method_type = MethodType::Callable;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };
            let methods = vec![method];

            let deployed_by = [0x00; 32];

            let program = Program::new(program_name, deployed_by, methods);

            assert!(program.is_err());
        }

        // A program with duplicate method names should fail.
        {
            let program_name = "test_program".to_string();
            let method_1 = {
                let method_name = "test_method".to_string();
                let method_type = MethodType::Callable;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };
            let method_2 = {
                let method_name = "test_method".to_string();
                let method_type = MethodType::Callable;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_FALSE(OP_FALSE),
                    Opcode::OP_FALSE(OP_FALSE),
                    Opcode::OP_FALSE(OP_FALSE),
                    Opcode::OP_FALSE(OP_FALSE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };
            let methods = vec![method_1, method_2];

            let deployed_by = [0x00; 32];

            let program = Program::new(program_name, deployed_by, methods);

            assert!(program.is_err());
        }

        // A program with all methods are internal should fail.
        {
            let program_name = "test_program".to_string();
            let method_1 = {
                let method_name = "test_method_1".to_string();
                let method_type = MethodType::Internal;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };
            let method_2 = {
                let method_name = "test_method_2".to_string();
                let method_type = MethodType::Internal;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };
            let methods = vec![method_1, method_2];

            let deployed_by = [0x00; 32];

            let program = Program::new(program_name, deployed_by, methods);

            assert!(program.is_err());
        }

        // Valid construction.
        {
            let program_name = "test_program".to_string();
            let method_1 = {
                let method_name = "test_method_1".to_string();
                let method_type = MethodType::Internal;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                    Opcode::OP_TRUE(OP_TRUE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };

            let method_2 = {
                let method_name = "test_method_2".to_string();
                let method_type = MethodType::Callable;
                let call_element_types = vec![CallElementType::U32, CallElementType::Account];
                let script = vec![
                    Opcode::OP_FALSE(OP_FALSE),
                    Opcode::OP_FALSE(OP_FALSE),
                    Opcode::OP_FALSE(OP_FALSE),
                    Opcode::OP_FALSE(OP_FALSE),
                ];

                ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
            };

            let methods = vec![method_1, method_2];

            let deployed_by = [0x00; 32];

            let program = Program::new(program_name, deployed_by, methods);

            assert!(program.is_ok());
        }
        Ok(())
    }

    #[test]
    fn program_compiler_test() -> Result<(), String> {
        let program_name = "test_program".to_string();
        let method_1 = {
            let method_name = "test_method_1".to_string();
            let method_type = MethodType::Internal;
            let call_element_types = vec![CallElementType::U32, CallElementType::Account];
            let script = vec![
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_TRUE(OP_TRUE),
                Opcode::OP_TRUE(OP_TRUE),
            ];

            ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
        };

        let method_2 = {
            let method_name = "test_method_2".to_string();
            let method_type = MethodType::Callable;
            let call_element_types = vec![CallElementType::U32, CallElementType::Account];
            let script = vec![
                Opcode::OP_FALSE(OP_FALSE),
                Opcode::OP_FALSE(OP_FALSE),
                Opcode::OP_FALSE(OP_FALSE),
                Opcode::OP_FALSE(OP_FALSE),
            ];

            ProgramMethod::new(method_name, method_type, call_element_types, script).unwrap()
        };

        let methods = vec![method_1, method_2];

        let deployed_by = [0x00; 32];

        let program = Program::new(program_name, deployed_by, methods).unwrap();

        let mut program_compiled_bytestream = program.compile().unwrap().into_iter();

        let program_decompiled = Program::decompile(&mut program_compiled_bytestream).unwrap();

        assert_eq!(program, program_decompiled);

        Ok(())
    }
}
