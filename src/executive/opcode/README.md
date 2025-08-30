# Opcodes
Cube uses an extended Bitcoin script with splicing, better memory management, and global state opcodes.

## Data push

| Opcode        | Bytecode  | Ops            | Input       | Output         | Description                                                                                        |
|:--------------|:----------|:---------------|:------------|:---------------|:---------------------------------------------------------------------------------------------------|
| OP_FALSE OP_0 | 0x00      | 1              | Nothing.    | (empty value)  | An empty array of bytes is pushed onto the stack.                                                  |
| OP_N/A        | 0x01-0x4b | 1 + (1 * Len)  | (special)   | data           | *Bytecode* number of following bytes pushed onto the stack.                                        |
| OP_PUSHDATA1  | 0x4c      | 1 + (1 * Len)  | (special)   | data           | The next byte contains the number of bytes to push onto the stack.                                 |
| OP_PUSHDATA2  | 0x4d      | 1 + (1 * Len)  | (special)   | data           | The next two bytes contain the number of bytes to be pushed onto the stack in little endian order. |
| OP_TRUE OP_1  | 0x51      | 1              | (special)   | 1              | The number 1 is pushed onto the stack.                                                             |
| OP_2-OP_16    | 0x52-0x60 | 1              | 2-16        | 1              | The number in the word name (2-16) is pushed onto the stack.                                       |

## Flow control

| Opcode        | Bytecode  | Ops | Input       | Output         | Description                                                                                        |
|:--------------|:----------|:----|:------------|:---------------|:---------------------------------------------------------------------------------------------------|
| OP_NOP        | 0x61      | 1   | Nothing.    | Nothing.       | Does nothing.                                                                                      |
| OP_RETURNERR  | 0x62      | 1   | Nothing.    | Return.        | Pops the top stack item and returns it as error message.                                           |
| OP_IF         | 0x63      | 1   | True/false  | Nothing.       | If the top stack value is not False, the statements are executed. The top stack value is removed.  |
| OP_NOTIF      | 0x64      | 1   | True/false  | Nothing.       | If the top stack value is False, the statements are executed. The top stack value is removed.      |
| OP_RETURNALL  | 0x65      | 1   | x1..xn      | Return.        | All stack items are popped and returned.                                                           |
| OP_RETURNSOME | 0x66      | 1   | x1..xn      | Return.        | Pops the stack item *count* and returns *count* number of items.                                   |
| OP_ELSE       | 0x67      | 1   | Nothing.    | Nothing.       | If the preceding OP_IF or OP_NOTIF or OP_ELSE was not executed then these statements are.          |
| OP_ENDIF      | 0x68      | 1   | Nothing.    | Nothing.       | Ends an if/else block. All blocks must end, or the transaction is invalid.                         |
| OP_VERIFY     | 0x69      | 1   | True/false  | Nothing/Fail   | Pops the top stack item and marks transaction as invalid if top stack value is not true.           |
| OP_FAIL       | 0x6a      | 1   | (special)   | Fail.          | Fails the entry.                                                                                   |

## Alstack Operations

| Opcode          | Bytecode | Ops | Input                   | Output                 | Description                                                                  |
|:----------------|:---------|:----|:------------------------|:-----------------------|:-----------------------------------------------------------------------------|
| OP_TOALTSTACK   | 0x6b     | 1   | x1                      | (alt)x1                | Puts the input onto the top of the alt stack. Removes it from the main stack. |
| OP_FROMALTSTACK | 0x6c     | 1   | (alt)x1                 | x1                     | Puts the input onto the top of the main stack. Removes it from the alt stack. |

## Stack Operations

| Opcode          | Bytecode | Ops | Input                   | Output                 | Description                                                                  |
|:----------------|:---------|:----|:------------------------|:-----------------------|:-----------------------------------------------------------------------------|
| OP_2DROP        | 0x6d     | 1   | x1 x2                   | Nothing                | Removes the top two stack items.                                             |
| OP_2DUP         | 0x6e     | 1   | x1 x2                   | x1 x2 x1 x2            | Duplicates the top two stack items.                                          |
| OP_3DUP         | 0x6f     | 1   | x1 x2 x3                | x1 x2 x3 x1 x2 x3      | Duplicates the top three stack items.                                        |
| OP_2OVER        | 0x70     | 1   | x1 x2 x3 x4             | x1 x2 x3 x4 x1 x2      | Copies the pair of items two spaces back in the stack to the front.          |
| OP_2ROT         | 0x71     | 1   | x1 x2 x3 x4 x5 x6       | x3 x4 x5 x6 x1 x2      | The fifth and sixth items back are moved to the top of the stack.            |
| OP_2SWAP        | 0x72     | 1   | x1 x2 x3 x4             | x3 x4 x1 x2            | Swaps the top two pairs of items.                                            |
| OP_IFDUP        | 0x73     | 1   | x                       | x / x x                | If the top stack value is not 0, duplicate it.                               |
| OP_DEPTH        | 0x74     | 1   | Nothing                 | <Stack size>           | Puts the number of stack items onto the stack.                               |
| OP_DROP         | 0x75     | 1   | x                       | Nothing                | Removes the top stack item.                                                  |
| OP_DUP          | 0x76     | 1   | x                       | x x                    | Duplicates the top stack item.                                               |
| OP_NIP          | 0x77     | 1   | x1 x2                   | x2                     | Removes the second-to-top stack item.                                        |
| OP_OVER         | 0x78     | 1   | x1 x2                   | x1 x2 x1               | Copies the second-to-top stack item to the top.                              |
| OP_PICK         | 0x79     | 1   | xn ... x2 x1 x0 <n>     | xn ... x2 x1 x0 xn     | The item n back in the stack is copied to the top.                           |
| OP_ROLL         | 0x7a     | 1   | xn ... x2 x1 x0 <n>     | ... x2 x1 x0 xn        | The item n back in the stack is moved to the top.                            |
| OP_ROT          | 0x7b     | 1   | x1 x2 x3                | x2 x3 x1               | The 3rd item down the stack is moved to the top.                             |
| OP_SWAP         | 0x7c     | 1   | x1 x2                   | x2 x1                  | The top two items on the stack are swapped.                                  |
| OP_TUCK         | 0x7d     | 1   | x1 x2                   | x2 x1 x2               | The item at the top of the stack is copied and inserted before the second-to-top item. |

## Splice

| Opcode          | Bytecode | Ops | Input             | Output       | Description                                                          |
|:----------------|----------|:----|:------------------|:-------------|:---------------------------------------------------------------------|
| OP_CAT          | 0x7e     | 2   | x1 x2             | out          | Concatenates two strings.                                            |
| OP_SPLIT        | 0x7f     | 2   | in index          | out out      | Splits the byte array into two stack items at the index.             |
| OP_LEFT         | 0x80     | 2   | in size           | out          | Keeps only characters left of the specified point in a string.       |
| OP_RIGHT        | 0x81     | 2   | in size           | out          | Keeps only characters right of the specified point in a string.      |
| OP_SIZE         | 0x82     | 2   | in                | in size      | Pushes the string length of the top element of the stack (without popping it). |

## Bitwise

| Opcode          | Bytecode | Ops | Input          | Output                                  | Description                                                                  |
|:----------------|:---------|:----|:---------------|:----------------------------------------|:-----------------------------------------------------------------------------|
| OP_INVERT       | 0x83     | 2   | in             | out                                     | Flips all of the bits in the input.                                          |
| OP_AND          | 0x84     | 2   | x1 x2          | out                                     | Boolean and between each bit in the inputs.                                  |
| OP_OR           | 0x85     | 2   | x1 x2          | out                                     | Boolean or between each bit in the inputs.                                   |
| OP_XOR          | 0x86     | 2   | x1 x2          | out                                     | Boolean exclusive or between each bit in the inputs.                         |
| OP_EQUAL        | 0x87     | 1   | x1 x2          | True / false                            | Returns 1 if the inputs are exactly equal, 0 otherwise.                      |
| OP_EQUALVERIFY  | 0x88     | 2   | x1 x2          | Nothing / fail                          | Same as OP_EQUAL, but runs OP_VERIFY afterward.                              |
| OP_REVERSE      | 0x89     | 3   | in             | out                                     | Pop the top item from the stack and reverses the byte order.                 |

## Arithmetic

| Opcode                | Bytecode | Ops | Input          | Output                                  | Description                                                                  |
|:----------------------|:---------|:----|:---------------|:----------------------------------------|:-----------------------------------------------------------------------------|
| OP_1ADD               | 0x8b     | 3   | in             | out                                     | 1 is added to the input.                                                     |
| OP_1SUB               | 0x8c     | 3   | in             | out                                     | 1 is subtracted from the input.                                              |
| OP_2MUL               | 0x8d     | 5   | in             | out                                     | The input is multiplied by 2.                                                |
| OP_2DIV               | 0x8e     | 5   | in             | out                                     | The input is divided by 2.                                                   |
| OP_ADDMOD             | 0x8f     | 3   | a b            | out                                     | a is added to b modulo MAX::U256.                                            |
| OP_MULMOD             | 0x90     | 3   | a b            | out                                     | a is multiplied by b modulo MAX::U256.                                       |
| OP_NOT                | 0x91     | 1   | in             | out                                     | If the input is 0 or 1, it is flipped. Otherwise the output will be 0.       |
| OP_0NOTEQUAL          | 0x92     | 1   | in             | out                                     | Returns 0 if the input is 0. 1 otherwise.                                    |
| OP_ADD                | 0x93     | 3   | a b            | out                                     | a is added to b.                                                             |
| OP_SUB                | 0x94     | 3   | a b            | out                                     | a is subtracted from b.                                                      |
| OP_MUL                | 0x95     | 5   | a b            | out                                     | a is multiplied by b.                                                        |
| OP_DIV                | 0x96     | 5   | a b            | out                                     | a is divided by b.                                                           |
| OP_LSHIFT             | 0x98     | 3   | a b            | out                                     | Shifts a left b bits.                                                        |
| OP_RSHIFT             | 0x99     | 3   | a b            | out                                     | Shifts a right b bits.                                                       |
| OP_BOOLAND            | 0x9a     | 1   | a b            | out                                     | If both a and b are not 0, the output is 1. Otherwise 0.                     |
| OP_BOOLOR             | 0x9b     | 1   | a b            | out                                     | If a or b is not 0, the output is 1. Otherwise 0.                            |
| OP_NUMEQUAL           | 0x9c     | 1   | a b            | out                                     | Returns 1 if the numbers are equal, 0 otherwise.                             |
| OP_NUMEQUALVERIFY     | 0x9d     | 2   | a b            | out                                     | Same as OP_NUMEQUAL, but runs OP_VERIFY afterward.                           |
| OP_NUMNOTEQUAL        | 0x9e     | 1   | a b            | out                                     | Returns 1 if the numbers are not equal, 0 otherwise.                         |
| OP_LESSTHAN           | 0x9f     | 1   | a b            | out                                     | Returns 1 if a is less than b, 0 otherwise.                                  |
| OP_GREATERTHAN        | 0xa0     | 1   | a b            | out                                     | Returns 1 if a is greater than b, 0 otherwise.                               |
| OP_LESSTHANOREQUAL    | 0xa1     | 1   | a b            | out                                     | Returns 1 if a is less than or equal to b, 0 otherwise.                      |
| OP_GREATERTHANOREQUAL | 0xa2     | 1   | a b            | out                                     | Returns 1 if a is greater than or equal to b, 0 otherwise.                   |
| OP_MIN                | 0xa3     | 1   | a b            | out                                     | Returns the smaller of a and b.                                              |
| OP_MAX                | 0xa4     | 1   | a b            | out                                     | Returns the larger of a and b.                                               |
| OP_WITHIN             | 0xa5     | 1   | x min max      | out                                     | Returns 1 if x is within the specified range (left-inclusive), 0 otherwise.  |

## Digest

| Opcode                | Bytecode | Ops                | Input            | Output  | Description                                                                  |
|:----------------------|:---------|:-------------------|:-----------------|:--------|:-----------------------------------------------------------------------------|
| OP_RIPEMD160          | 0xa6     | 30                 | preimage         | hash    | The input is hashed using RIPEMD-160.                                        |
| OP_SHA1               | 0xa7     | 30                 | preimage         | hash    | The input is hashed using SHA-1.                                             |
| OP_SHA256             | 0xa8     | 42                 | preimage         | hash    | The input is hashed using SHA-256.                                           |
| OP_HASH160            | 0xa9     | 72                 | preimage         | hash    | The input is hashed twice: first with SHA-256 and then with RIPEMD-160.      |
| OP_HASH256            | 0xaa     | 84                 | preimage         | hash    | The input is hashed two times with SHA-256.                                  |
| OP_TAGGEDHASH         | 0xab     | 42                 | preimage tag     | hash    | The input is hashed with a domain seperation tag.                            |
| OP_BLAKE2BVAR         | 0xac     | 10 + (1 * outsize) | preimage outsize | hash    | The input is hashed using Blake2b with the output size from stack.           |
| OP_BLAKE2SVAR         | 0xad     | 10 + (1 * outsize) | preimage outsize | hash    | The input is hashed using Blake2s with the output size from stack.           |

## Secp

| Opcode                    | Bytecode | Ops            | Input            | Output              | Description                                                                |
|:--------------------------|:---------|:---------------|:-----------------|:--------------------|:---------------------------------------------------------------------------|
| OP_SECPSCALARADD          | 0xae     | 10             | scalar scalar    | scalar              | Adds two secp scalars.                                                     |
| OP_SECPSCALARMUL          | 0xaf     | 10             | scalar scalar    | scalar              | Multiplies two secp scalars.                                               |
| OP_SECPPOINTADD           | 0xb0     | 50             | point point      | point               | Adds two secp points.                                                      |
| OP_SECPPOINTMUL           | 0xb1     | 50             | point scalar     | point               | Multiplies a secp point by a secp scalar.                                  |
| OP_PUSHSECPGENERATORPOINT | 0xb2     | 50             | Nothing          | point               | Pushes generator point into stack.                                         |
| OP_ISZEROSECPSCALAR       | 0xb3     | 50             | scalar           | scalar true/false   | Returns whether the scalar is zero.                                        |
| OP_ISINFINITESECPPOINT    | 0xb4     | 50             | point            | point true/false    | Returns whether the point is at infinity.                                  |

## Digital signatures

| Opcode                   | Bytecode | Ops                | Input                 | Output            | Description                                                          |
|:-------------------------|:---------|:-------------------|:----------------------|:------------------|:---------------------------------------------------------------------|
| OP_CHECKSCHNORRSIG       | 0xb5     | 100                | sig msg key           | True/false        | Checks a schnorr signature according to the 'Cube/challenge' tag. |
| OP_CHECKSCHNORRSIGBIP340 | 0xb6     | 100                | sig msg key           | True/false        | Checks a schnorr signature according to the 'BIP0340/challenge' tag. |
| OP_CHECKBLSSIG           | 0xb7     | 100                | sig msg key           | True/false        | Checks a BLS signature against a key and a message.                  |
| OP_CHECKBLSSIGAGG        | 0xb8     | 100 + (50 * count) | sig [msg] [key] count | True/false        | Checks a BLS aggregate signature against a set of keys and messages. |

## Call info

| Opcode         | Bytecode | Ops | Input                | Output                 | Description                                                                      |
|:---------------|:---------|:----|:---------------------|:-----------------------|:---------------------------------------------------------------------------------|
| OP_CALLER      | 0xb9     | 1   | -                    | True/false id          | Pushes the caller type and id to the stack.                                      |
| OP_OPSBUDGET   | 0xba     | 1   | -                    | out                    | Pushes the ops budget into stack.                                                |
| OP_OPSCOUNTER  | 0xbb     | 1   | -                    | out                    | Pushes the number of ops spent into stack.                                       |
| OP_OPSPRICE    | 0xbc     | 1   | -                    | out                    | Pushes the ops price into stack.                                                 |
| OP_TIMESTAMP   | 0xbd     | 1   | -                    | out                    | Pushes the call timestamp into stack.                                            |

## Call 

| Opcode         | Bytecode | Ops | Input                 | Output                 | Description                                                                     |
|:---------------|:---------|:----|:----------------------|:-----------------------|:--------------------------------------------------------------------------------|
| OP_CALL        | 0xbe     | 5   | [args] count index    | Return.                | Calls an internal contract method.                                              |
| OP_CALLEXT     | 0xbf     | 50  | [args] count index id | Return.                | Calls an external contract method.                                              |

## Coin 

| Opcode           | Bytecode | Ops              | Input                   | Output        | Description                                                             |
|:-----------------|:---------|:-----------------|:------------------------|:--------------|:------------------------------------------------------------------------|
| OP_BALANCE       | 0xc0     | 1                | kind destination        | out/fail.     | Pops the kind and pushes the contract or account balance into stack.    |
| OP_TRANSFER      | 0xc1     | 10               | kind destination amount | Nothing/fail. | Pops the kind and transfers sats to the account or the contract.        |

## Shadowing 

| Opcode                    | Bytecode | Ops              | Input                 | Output                 | Description                                             |
|:--------------------------|:---------|:-----------------|:-----------|:-----------------------|:-------------------------------------------------------------------|
| OP_SHADOW_ALLOC           | 0xc2     | 1                | key        | Nothing/fail.          | Allocates within the contract shadow space an account.             |
| OP_SHADOW_ALLOC_UP        | 0xc3     | 1                | key amount | Nothing/fail.          | Increases the shadow space allocation of an account.               |
| OP_SHADOW_ALLOC_DOWN      | 0xc4     | 10               | key amount | Nothing/fail.          | Decreases the shadow space allocation of an account.               |
| OP_SHADOW_ALLOC_UP_ALL    | 0xc5     | 10               | amount     | Nothing/fail.          | Proportionally increases shadow space allocations of all accounts. |
| OP_SHADOW_ALLOC_DOWN_ALL  | 0xc6     | 10               | amount     | Nothing/fail.          | Proportionally decreases shadow space allocations of all accounts. |

## Storage

| Opcode         | Bytecode | Ops | Input                | Output                 | Description                                                                      |
|:---------------|:---------|:----|:---------------------|:-----------------------|:---------------------------------------------------------------------------------|
| OP_SWRITE      | 0xc7     | 50  | x1 x2                | x1                     | Pops the storage key and value, and writes the value to the contract's storage.  |
| OP_SREAD       | 0xc8     | 50  | x1                   | x1                     | Pops the storage key, and reads the value from the contract's storage.           |

## Memory

| Opcode         | Bytecode | Ops | Input                | Output                 | Description                                                                      |
|:---------------|:---------|:----|:---------------------|:-----------------------|:---------------------------------------------------------------------------------|
| OP_MWRITE      | 0xca     | 5   | x1 x2                | x1                     | Pops the memory key and value, and writes the value to the contract's memory.    |
| OP_MREAD       | 0xcb     | 5   | x1                   | x1                     | Pops the memory key, and reads the value from the contract's memory.             |
| OP_MFREE       | 0xcc     | 1   | x1                   | x1                     | Pops the memory key, and frees the key/value from the contract's memory.         |
