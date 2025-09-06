#[cfg(test)]
mod stack_tests {
    use cube::executive::{
        exec::caller::Caller,
        opcode::opcodes::{
            altstack::{op_fromaltstack::OP_FROMALTSTACK, op_toaltstack::OP_TOALTSTACK},
            arithmetic::op_add::OP_ADD,
            bitwise::op_equalverify::OP_EQUALVERIFY,
            flow::{
                op_else::OP_ELSE, op_endif::OP_ENDIF, op_fail::OP_FAIL, op_if::OP_IF,
                op_verify::OP_VERIFY,
            },
            push::{
                op_2::OP_2, op_3::OP_3, op_4::OP_4, op_5::OP_5, op_6::OP_6, op_7::OP_7, op_8::OP_8,
                op_false::OP_FALSE, op_true::OP_TRUE,
            },
            splice::op_cat::OP_CAT,
        },
        stack::{
            stack::Stack,
            stack_error::StackError,
            stack_holder::StackHolder,
            stack_item::StackItem,
            stack_uint::{StackItemUintExt, StackUint},
        },
    };

    #[test]
    fn stack_test() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack.
        let mut stack_holder = StackHolder::new(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
        )?;

        // Push 0xdeadbeef
        let _ = stack_holder.push(StackItem::new(vec![0xde, 0xad, 0xbe, 0xef]));

        // Push 0xdead
        let _ = stack_holder.push(StackItem::new(vec![0xde, 0xad]));

        // Push 0xbeef
        let _ = stack_holder.push(StackItem::new(vec![0xbe, 0xef]));

        // OP_TOALTSTACK
        OP_TOALTSTACK::execute(&mut stack_holder)?;

        // OP_TOALTSTACK
        OP_TOALTSTACK::execute(&mut stack_holder)?;

        // OP_FROMALTSTACK
        OP_FROMALTSTACK::execute(&mut stack_holder)?;

        // OP_FROMALTSTACK
        OP_FROMALTSTACK::execute(&mut stack_holder)?;

        // OP_CAT
        OP_CAT::execute(&mut stack_holder)?;

        // OP_EQUALVERIFY
        OP_EQUALVERIFY::execute(&mut stack_holder)?;

        Ok(())
    }

    #[test]
    fn arithmetic_addition_test() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack.
        let mut stack_holder = StackHolder::new(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
        )?;

        // Test 0 + 1 = 1;
        {
            // Push 1
            let item = StackItem::from_stack_uint(StackUint::from(1));
            assert_eq!(item.bytes(), vec![0x01]);
            let _ = stack_holder.push(item);

            // Push 1
            let item = StackItem::from_stack_uint(StackUint::from(1));
            assert_eq!(item.bytes(), vec![0x01]);
            let _ = stack_holder.push(item);

            // Push 0
            let item = StackItem::from_stack_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // OP_ADD
            OP_ADD::execute(&mut stack_holder)?;

            // OP_VERIFY to check if addition result is equal to 0.
            OP_VERIFY::execute(&mut stack_holder)?;

            // OP_EQUALVERIFY to check if addition result is equal to 0.
            OP_EQUALVERIFY::execute(&mut stack_holder)?;

            // Stack must be empty.
            assert_eq!(stack_holder.stack_items_count(), 0);
        }

        // Test 0 + 0 = 0;
        {
            // Push 0
            let item = StackItem::from_stack_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // Push 0
            let item = StackItem::from_stack_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // Push 0
            let item = StackItem::from_stack_uint(StackUint::from(0));
            assert_eq!(item.bytes().len(), 0);
            let _ = stack_holder.push(item);

            // OP_ADD
            OP_ADD::execute(&mut stack_holder)?;

            // OP_VERIFY to check if addition result is equal to 0.
            OP_VERIFY::execute(&mut stack_holder)?;

            // OP_EQUALVERIFY to check if addition result is equal to 0.
            OP_EQUALVERIFY::execute(&mut stack_holder)?;

            // Stack must be empty.
            assert_eq!(stack_holder.stack_items_count(), 0);
        }

        // Test 100 + 50 = 150;
        {
            // Push 150
            let item = StackItem::from_stack_uint(StackUint::from(150));
            assert_eq!(item.bytes(), vec![0x96]);
            let _ = stack_holder.push(item);

            // Push 100
            let item = StackItem::from_stack_uint(StackUint::from(100));
            assert_eq!(item.bytes(), vec![0x64]);
            let _ = stack_holder.push(item);

            // Push 50
            let item = StackItem::from_stack_uint(StackUint::from(50));
            assert_eq!(item.bytes(), vec![0x32]);
            let _ = stack_holder.push(item);

            // OP_ADD
            OP_ADD::execute(&mut stack_holder)?;

            // OP_VERIFY to check if additon was successful.
            OP_VERIFY::execute(&mut stack_holder)?;

            // OP_EQUALVERIFY to check if addition result is equal to 150.
            OP_EQUALVERIFY::execute(&mut stack_holder)?;

            // Stack must be empty.
            assert_eq!(stack_holder.stack_items_count(), 0);
        }

        Ok(())
    }

    #[test]
    fn flow_test() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with true.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::true_item()],
        )?;

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_2
        OP_2::execute(&mut stack_holder)?;

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_3
        OP_3::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // Expected stack after execution ends with 2 on top.
        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(2))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        Ok(())
    }

    #[test]
    fn nested_flow_test() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with true.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::true_item()],
        )?;

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_FALSE
        OP_FALSE::execute(&mut stack_holder)?;

        // Nested OP_IF/OP_ELSE/OP_ENDIF

        {
            // OP_IF
            OP_IF::execute(&mut stack_holder)?;

            // OP_2
            OP_2::execute(&mut stack_holder)?;

            // OP_ELSE
            OP_ELSE::execute(&mut stack_holder)?;

            // OP_3 (this will be executed!)
            OP_3::execute(&mut stack_holder)?;

            // OP_ENDIF
            OP_ENDIF::execute(&mut stack_holder)?;
        }

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_4
        OP_4::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // Expected stack after execution ends with 3 on top.
        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(3))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_2() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with true.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::true_item()],
        )?;

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_FALSE
        OP_FALSE::execute(&mut stack_holder)?;

        // Nested OP_IF/OP_ELSE/OP_ENDIF

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_2
        OP_2::execute(&mut stack_holder)?;

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_3
        OP_3::execute(&mut stack_holder)?;

        // Another nesting

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_4 (this will be executed!)
        OP_4::execute(&mut stack_holder)?;

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_5
        OP_5::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_4
        OP_4::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // Expected stack after execution ends with 4 on top.
        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(4))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_3() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with false.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::false_item()],
        )?;

        // OP_IF
        OP_IF::execute(&mut stack_holder)?;

        // OP_FALSE
        OP_FALSE::execute(&mut stack_holder)?;

        // Nested OP_IF/OP_ELSE/OP_ENDIF

        {
            // OP_IF
            OP_IF::execute(&mut stack_holder)?;

            // OP_2
            OP_2::execute(&mut stack_holder)?;

            // OP_ELSE
            OP_ELSE::execute(&mut stack_holder)?;

            // OP_3
            OP_3::execute(&mut stack_holder)?;

            // Another nesting
            {
                // OP_IF
                OP_IF::execute(&mut stack_holder)?;

                // OP_4 (this will be executed!)
                OP_4::execute(&mut stack_holder)?;

                // OP_ELSE
                OP_ELSE::execute(&mut stack_holder)?;

                // OP_5
                OP_5::execute(&mut stack_holder)?;

                // OP_ENDIF
                OP_ENDIF::execute(&mut stack_holder)?;
            }

            // OP_ENDIF
            OP_ENDIF::execute(&mut stack_holder)?;
        }

        // OP_ELSE
        OP_ELSE::execute(&mut stack_holder)?;

        // OP_4
        OP_4::execute(&mut stack_holder)?;

        // OP_ENDIF
        OP_ENDIF::execute(&mut stack_holder)?;

        // Expected stack after execution ends with 4 on top.
        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(4))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_4() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with false.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::false_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_TRUE::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_2::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_5::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_6::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_4::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(6))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_5() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with false.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::false_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_TRUE::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_2::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_5::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_7::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_8::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_4::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(8))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_6() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with true.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::true_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_2::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_3::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_4::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_5::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(5))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_7() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with true.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::true_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_2::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_3::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_4::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_6::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(6))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_8() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with true.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::true_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_2::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_3::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(3))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_9() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with false.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::false_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_TRUE::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(1))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }

    #[test]
    fn nested_flow_test_10() -> Result<(), StackError> {
        let internal_ops_counter = 0;
        let external_ops_counter = 0;

        let timestamp = 1715619200;

        // Initialize stack with false.
        let mut stack_holder = StackHolder::new_with_items(
            Caller::new_account([0; 32]),
            [0; 32],
            timestamp,
            0,
            50,
            1,
            internal_ops_counter,
            external_ops_counter,
            vec![StackItem::false_item()],
        )?;

        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_TRUE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_TRUE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FALSE::execute(&mut stack_holder)?;
        OP_IF::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_2::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ELSE::execute(&mut stack_holder)?;
        OP_FAIL::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;
        OP_ENDIF::execute(&mut stack_holder)?;

        let expected_stack =
            Stack::new_with_items(vec![StackItem::from_stack_uint(StackUint::from(2))]);

        // Assert that the stack is equal to the expected stack.
        assert_eq!(stack_holder.stack().clone(), expected_stack);

        // Flows encountered must be empty.
        assert_eq!(stack_holder.flow_encounters_len(), 0);

        Ok(())
    }
}
