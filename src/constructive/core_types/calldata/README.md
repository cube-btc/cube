# Calldata
Cube supports 10 calldata types that `Contracts` can accept as parameters:

| Calldata Type    | Decoded As              | Description                                                                                 | CPE Bitsize   | Stack Bytesize |
|:-----------------|:------------------------|:--------------------------------------------------------------------------------------------|:--------------|:---------------|
| U8               | u8                      | Represents values ranging from 0 to 255.                                                    | 8 bits        | 0-1 bytes      |
| U16              | u16                     | Represents values ranging from 0 to 65,535.                                                 | 16 bits       | 0-2 bytes      |
| U32              | `MaybeCommon<ShortVal>` | Represents compact and possibly common values ranging from 0 to 4,294,967,295.              | 7-35 bits     | 0-4 bytes      |
| U64              | `MaybeCommon<LongVal>`  | Represents compact and possibly common values ranging from 0 to 18,446,744,073,709,551,615. | 7-68 bits     | 0-8 bytes      |
| Bool             | bool                    | Represents a boolean; either true or false.                                                 | 1 bit         | 0-1 bytes      |
| Bytes1-256       | [u8; 1-256]             | Represents a fixed-length byte array, ranging from 1 to 256 bytes.                          | 8-2048 bits   | 1-256 bytes    |
| Varbytes         | Vec<u8>                 | Represents a dynamic-length byte array, ranging from 0 to 4095 (max stack size) bytes.      | 12-32772 bits | 0-4095 bytes   |
| Account          | `Account`               | Represents a possibly registered `Account`.                                                 | 11-257 bits   | 32 bytes       |
| Contract         | `Contract`              | Represents a deployed `Contract`.                                                           | 10-34 bits    | 32 bytes       |
| Payable          | `MaybeCommon<ShortVal>` | Represents a Bitcoin payment.                                                               | 7-35 bits     | 0-4 bytes      |