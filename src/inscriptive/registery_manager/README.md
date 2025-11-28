# Registery Manager 👮‍♂️

Local storage manager for civil registration affairs for permanent Cube residents (accounts) and deployed smart contracts.

## Ranking

The registry manager ranks accounts and contracts based on their engagement level with Cube. Both accounts and contracts are assigned a call counter that determines their rank. Rankings are cached in-memory for fast access and reference, using Airly DA compression to significantly reduce the on-chain footprint associated with account/contract referencing, trading off increased memory requirements for improved DA efficiency.

### Account Ranking

Accounts are ranked based on transaction frequency. Each interaction—whether a vanilla value transfer or a smart contract call—increments the account's call counter by one, which in turn affects its rank.

### Contract Ranking

Contracts are ranked based on how frequently they are called by accounts. Each invocation increments the contract's call counter by one, which then affects its rank.
