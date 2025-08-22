# Contract Coin Holder

A state management system for handling contract BTC balances and shadow space allocations within the Cube virtual machine.

## Overview

The system operates on two fundamental entities: **accounts** and **contracts**. Contracts are an aboveley construct that breaks down into belowley accounts through a shadowing mechanism.

Each contract is allocated a shadow space bound by its actual BTC balance. This shadow space mirrors and projects the contract's real value into its associated accounts through controlled allocations. While the contract's actual balance remains immutable, the shadow space allows us to translate contract logic into shadow logic - a technique that provides granular control over fund distribution without moving the underlying Bitcoin.

This shadowing technique enables contracts to maintain their real value while projecting virtual representations into account allocations that we can manipulate and control.

## Core Concept

- **Contract Balance**: The immutable real value held by the aboveley contract construct
- **Shadow Space**: The belowley projection mechanism that mirrors the contract's real value
- **Account Allocations**: The shadowed representations of the contract's value distributed across belowley accounts

## Data Schema

The system organizes data in a nested hierarchy where each contract maintains both its actual balance and a virtual shadow space that mirrors this balance through account allocations.

```
CONTRACT_ID → ContractBody
├── balance: Contract's actual BTC balance amount
└── shadow_space: ContractShadowSpace
    ├── allocs_sum: Total allocated amount (cannot exceed contract balance)
    └── allocs: Account allocation mapping
        ├── ACCOUNT_ID → BTC_VALUE (Individual account's allocated portion)
        ├── ACCOUNT_ID → BTC_VALUE (Individual account's allocated portion)
        └── ...
```

### Key Relationships

- **Contract Balance**: Each contract holds an actual BTC balance that serves as the upper limit
- **Shadow Space**: A virtual representation where the `allocs_sum` mimics the contract's balance through distributed account allocations
- **Balance Constraint**: The total of all account allocations (`allocs_sum`) cannot exceed the contract's actual balance
- **Account Allocations**: Individual accounts claim portions of the contract's balance through their allocated BTC values

## Key Operations

- **Register Contracts**: Initialize new contracts with zero balance and empty shadow space
- **Allocate Space**: Create allocation space for accounts within contract shadow spaces
- **Modify Allocations**: Increase or decrease account BTC balance values
- **Query Balances**: Check contract balances and account allocations

## State Management

The system uses three layers:
1. **Ephemeral States**: Transaction processing with rollback support
2. **In-Memory Cache**: Fast access to contract states
3. **Persistent Storage**: Sled database for durability

**Note**: The system currently caches all contract states in-memory for maximum performance. As the number of contracts grows, this approach may require refactoring to implement selective caching or lazy loading strategies.

## Safety Features

- **Balance Constraints**: Total allocations cannot exceed contract balance
- **Non-Negative Values**: Account allocations cannot go below zero
- **Atomic Operations**: All changes are atomic with full rollback support
- **Error Handling**: Comprehensive error types for various failure scenarios

## Use Cases

- **DeFi Protocols**: Liquidity pools, staking systems, yield farming
- **Multi-Signature Wallets**: Shared fund management
- **Smart Contract Financial Primitives**: Virtual balance management

This module provides the foundation for sophisticated financial applications on the Cube platform. 