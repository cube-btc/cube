# Valtypes
Cube employs 5 specialized value types to minimize Bitcoin block space consumption, thereby optimizing data availability (DA) efficiency. The protocol is designed to ensure that transaction summary data is encoded in a highly compact form before being published in the `Payload` field of on-chain rollup state transitions.

The value types are defined as follows:

| Value Type           | Description                                                                     |
|----------------------|---------------------------------------------------------------------------------|
| AtomicVal            | A highly compact integer representation ranging from 0 to 255.                  |
| ShortVal             | A flexible integer representation ranging from 0 to 4,294,967,295.              |
| MaybeCommon ShortVal | A possibly common `ShortVal`.                                                   |
| LongVal              | A scalable integer representation ranging from 0 to 18,446,744,073,709,551,615. |
| MaybeCommon LongVal  | A possibly common `LongVal`.                                                    |

## AtomicVal
`AtomicVal` is a highly compact unsigned integer representation ranging from 0 to 255. It dynamically determines the number of bits it needs to collect based on an upper bound.

| AtomicVal Tiers | Description                        | Upper Bound Range | Bitsize |
|-----------------|------------------------------------|-------------------|---------|
| b1              | Represents values from 0 to 1.     | 0 <= x <= 1       | 1 bit   |
| b2              | Represents values from 0 to 3.     | 1 <= x <= 3       | 2 bits  |
| b3              | Represents values from 0 to 7.     | 3 <= x <= 7       | 3 bits  |
| b4              | Represents values from 0 to 15.    | 7 <= x <= 15      | 4 bits  |
| b5              | Represents values from 0 to 31.    | 15 <= x <= 31     | 5 bits  |
| b6              | Represents values from 0 to 63.    | 31 <= x <= 63     | 6 bits  |
| b7              | Represents values from 0 to 127.   | 63 <= x <= 127    | 7 bits  |
| b8              | Represents values from 0 to 255.   | 127 <= x <= 255   | 8 bits  |

`AtomicVal` is used for encoding very small values, such as `Contract` call methods.

For an average `Contract` with four callable methods, `AtomicVal` falls into the b2 tier, and consume only 2 bits. Compared to traditional EVM `function selectors`, which use 4 bytes, this results in a savings of 30 bits per `Entry` in the `Payload`, translating to approximately ~0.93 vBytes in block space.

## ShortVal

`ShortVal` is a compact unsigned integer representation ranging from 0 (inclusive) to 4,294,967,295 (inclusive). It is used for representing small values such as contract or account registry indexes. `ShortVal` consumes a varying amount of bit space.

| ShortVal Tiers | Description                                | Bitsize |
|----------------|--------------------------------------------|---------|
| U8             | Represents values from 0 to 255.           | 10 bits |
| U16            | Represents values from 0 to 65,535.        | 18 bits |
| U24            | Represents values from 0 to 16,777,215.    | 26 bits |
| U32            | Represents values from 0 to 4,294,967,295. | 34 bits |

## MaybeCommon ShortVal

`MaybeCommon ShortVal` is a possibly common `ShortVal` representation. This adds one bit of overhead to the bit size of `ShortVal` to determine whether the encoded value is common.

A common value of `1000`, for instance, would fall into the U16 tier of `ShortVal` and consume 18 bits. When represented with `MaybeCommon ShortVal`, it consumes only 7 bits, resulting in a savings of 11 bits per `Entry` in the `Payload`, translating to approximately ~0.34 vBytes of block space savings.

| MaybeCommon ShortVal Tiers  | Description                                | Bitsize |
|-----------------------------|--------------------------------------------|---------|
| Common                      | Represents a list of 64 common values.     | 7 bits  |
| Uncommon U8                 | Represents values from 0 to 255.           | 11 bits |
| Uncommon U16                | Represents values from 0 to 65,535.        | 19 bits |
| Uncommon U24                | Represents values from 0 to 16,777,215.    | 27 bits |
| Uncommon U32                | Represents values from 0 to 4,294,967,295. | 35 bits |

## LongVal

`LongVal` is a compact unsigned integer representation ranging from 0 (inclusive) to 18,446,744,073,709,551,615 (inclusive). It is used for representing large values such as contract parameters. `LongVal` consumes a varying amount of bit space.

| LongVal Tiers| Description                                            | Bitsize|
|--------------|--------------------------------------------------------|---------|
| U8           | Represents values from 0 to 255.                       | 11 bits |
| U16          | Represents values from 0 to 65,535.                    | 19 bits |
| U24          | Represents values from 0 to 16,777,215.                | 27 bits |
| U32          | Represents values from 0 to 4,294,967,295.             | 35 bits |
| U40          | Represents values from 0 to 1,099,511,627,775.         | 43 bits |
| U48          | Represents values from 0 to 281,474,976,710,655.       | 51 bits |
| U56          | Represents values from 0 to 72,057,594,037,927,935.    | 59 bits |
| U64          | Represents values from 0 to 18,446,744,073,709,551,615.| 67 bits |

## MaybeCommon LongVal

`MaybeCommon LongVal` is a possibly common `LongVal` representation. This adds one bit of overhead to the bit size of `LongVal` to determine whether the encoded value is common.

| MaybeCommon LongVal Tiers | Description                                            | Bitsize|
|---------------------------|--------------------------------------------------------|---------|
| Common                    | Represents a list of 64 common values.                 | 8 bits  |
| Uncommon U8               | Represents values from 0 to 255.                       | 12 bits |
| Uncommon U16              | Represents values from 0 to 65,535.                    | 20 bits |
| Uncommon U24              | Represents values from 0 to 16,777,215.                | 28 bits |
| Uncommon U32              | Represents values from 0 to 4,294,967,295.             | 36 bits |
| Uncommon U40              | Represents values from 0 to 1,099,511,627,775.         | 44 bits |
| Uncommon U48              | Represents values from 0 to 281,474,976,710,655.       | 52 bits |
| Uncommon U56              | Represents values from 0 to 72,057,594,037,927,935.    | 60 bits |
| Uncommon U64              | Represents values from 0 to 18,446,744,073,709,551,615.| 68 bits |
