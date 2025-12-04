# Airly Compression
`Airly Compression` is a custom-tailored algorithm designed to optimize data availability (DA) efficiency for Bitcoin.

`Airly Compression` allows Cube to pack as many transactions as possible into a Bitcoin block, enabling it to handle significantly more transactions compared to zkEVM and EVM. By optimizing transaction encoding, indexing, and signature aggregation, Cube achieves higher throughput and lower transaction costs, making it a highly scalable solution for decentralized Bitcoin applications.

## VM Comparsion
| VM Type | Encoding                        | Scope      | Indexing        | Nonce     | Gas Price/Limit | Calldata     | Signature      | Error-handling |
|:--------|:--------------------------------|:-----------|:----------------|-----------|:----------------|:-------------|:---------------|:---------------|
| Cube    | Compact-payload-encoding (CPE)  | Bit-level  | Rank-based      | -         | Negligible      | Non-prefixed | BLS-aggregated | Assertions     |
| zkEVM   | Recursive-length prefix (RLP)   | Byte-level | Registery-based | Present   | Present         | Prefixed     | ZK-aggregated  | Failures       |
| EVM     | Recursive-length prefix (RLP)   | Byte-level | -               | Present   | Present         | Prefixed     | 65 bytes       | Failures       |

`Airly Compression` is composed of 9 key techniques:

#### 1. Bit-level Encoding
Cube uses bit-level `CPE` encoding for transaction and value types, unlike the standard byte-level `RLP` encoding used by zkEVM and EVM. 

While `RLP` encoding requires 4 bytes for `u32` and 8 bytes for `u64`, `CPE` encodes these values in smaller units, allowing more data to fit into the same space. This results in savings of 1-3 bytes for `u32` and 1-7 bytes for `u64`, with only a 2-3 bit overhead.

Bit-level encoding is impractical for zkEVMs due to the increased complexity in generating zero-knowledge proofs (ZKPs), as it demands more precise data handling.

See [Valtype](https://github.com/cube-vm/cube/tree/main/src/constructive/valtype).

#### 2. Common Value Lookup
Cube uses a lookup table to efficiently encode commonly used values like 100, 5,000, and 10,000,000. This method significantly reduces byte usage when contracts with fewer decimal places are called with these values. By leveraging the lookup table to encode frequent patterns, Cube minimizes DA overhead at scale. 

See [CommonVal](https://github.com/cube-vm/cube/blob/main/src/constructive/valtype/maybe_common/common_val.rs).

#### 3. Rank-based Indexing
Cube indexes `Accounts` and `Contracts` based on how frequently they transact, rather than when they are registered. Each time an `Account` initiates a transaction or a `Contract` is called, their rank is incremented by one.

This rank-based indexing system is cached and managed at the memory level, ensuring that frequently used contracts—such as AMM pools or Tether—consume only ~1 byte, compared to zkEVM’s 4 bytes and EVM’s 20 bytes.

#### 4. Non-prefixed Calldata
Cube maps calldata items directly to pre-defined types with known lengths, eliminating the prefix overhead for calldata. In contrast, the EVM requires calldata to be prefixed with an `RLP` encoding, adding 1-2 bytes overhead.

See [Calldata](https://github.com/cube-vm/cube/tree/main/src/constructive/calldata).

#### 5. Compact Call Method
Cube decodes `Contract` call methods through a varying bitsize `AtomicVal`.

In the case of an average `Contract` with four callable methods, `AtomicVal` would consume only 2 bits. In contrast, traditional EVM function selectors require 4 bytes. This results in a savings of 30 bits per `Entry` in the `Payload`, translating to an approximate ~0.93 vBytes of block space savings.

See [Atomicval](https://github.com/cube-vm/cube/tree/main/src/constructive/valtype#atomicval).

#### 6. Signature Aggregation
Cube aggregates transaction signatures using `MuSig2`, resulting in a constant 64-byte aggregated signature, instead of using ZKPs, which typically take up around 500 bytes. This results in a saving of 436 bytes per block compared to zkEVMs. 

See [Musig-nested-NOIST](https://blog.brollup.org/covenant-emulation-with-musig-nested-noist-784d428c7446).

#### 7. Nonceless
Cube omits nonce field from the transaction encoding scheme to track internal transaction states. Since rollup state transitions are externally chained, the requirement for internal chaining is eliminated.

See [Entry](https://github.com/cube-vm/cube/tree/main/src/constructive/entry).

#### 8. Fluctuating Gas-limit
Cube replaces `Gas` with `Ops` and introduces a fluctuating `Ops limit` where only the overhead value is encoded, if ever present. 

See [Entry](https://github.com/cube-vm/cube/tree/main/src/constructive/entry).

#### 9. Assertations
In Cube, transactions are asserted, meaning that only valid transactions are included in blocks. Failed transactions are never recorded, resulting in a cleaner state and fewer invalid operations. In contrast, both zkEVM and Ethereum allow failed transactions to end up in blocks, which increases overhead and reduces overall efficiency. This means Cube achieves an overall 5% historical block space savings in comparison.

## Savings

#### AMM Swap
Taking an average AMM contract swap as an example, Cube consumes only around ~10.7 bytes (~2.67 vBytes) of block space. In comparison, zkEVM requires 33 bytes, resulting in an approximate savings of 5.62 vBytes of block space. As a result, Cube can handle about 3.1x times more AMM swaps than a zkEVM clone on Bitcoin, and approximately 10.5x times more than a standard EVM.

| VM Type | From Account | To Contract | Value    | Nonce    | Gas Price/Limit | Call Method | Calldata   | Signature  | Size       | Efficiency | TPS |
|:--------|:-------------|:------------|:---------|:---------|:----------------|:------------|:-----------|:-----------|:-----------|:-----------|:----|
| Cube    | ~3 bytes     | ~10 bits    | -        | -        | 2 bits          | 2 bits      | 6 bytes    | Negligible | 10.7 bytes | 10.5x      | 634 |
| zkEVM   | 4 bytes      | 4 bytes     | 1 byte   | ~3 bytes | ~8 bytes        | 4 bytes     | 9 bytes    | Negligible | 33 bytes   | 3.3x       | 202 |
| EVM     | -            | 20 bytes    | 1 byte   | ~3 bytes | ~8 bytes        | 4 bytes     | 9 bytes    | 65 bytes   | 110 bytes  | 1x         | 60  |


#### Token Transfer
Taking a standard token transfer as an example, Cube consumes only around ~9.7 bytes (~2.42 vBytes) of block space. In comparison, zkEVM requires 33 bytes, resulting in an approximate savings of 5.8 vBytes of block space. As a result, Cube can handle about 3.5x times more token transfers than a zkEVM clone on Bitcoin, and approximately 13.2x times more than a standard EVM.

| VM Type | From Account | To Contract | Value    | Nonce    | Gas Price/Limit | Call Method | Calldata   | Signature  | Size       | Efficiency | TPS |
|:--------|:-------------|:------------|:---------|:---------|-----------------|:------------|:-----------|:-----------|:-----------|:-----------|:----|
| Cube    | ~3 bytes     | ~10 bits    | -        | -        | 2 bits          | 2 bits      | 5 bytes    | Negligible | 9.7 bytes  | 13.2x      | 701 |
| zkEVM   | 4 bytes      | 4 bytes     | 1 byte   | ~3 bytes | ~8 bytes        | 4 bytes     | 9 bytes    | Negligible | 33 bytes   | 3.8x       | 202 |
| EVM     | -            | 20 bytes    | 1 byte   | ~3 bytes | ~8 bytes        | 4 bytes     | 25 bytes   | 65 bytes   | 126 bytes  | 1x         | 52  |


> [!NOTE]
> Comparisons excludes further savings from (2) common value lookup, (6) signature aggregation, and (9) assertations. Factoring in these optimizations, the efficiency is projected to surpass the estimate.