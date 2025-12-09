![Cube](https://i.ibb.co/KjnGsD7L/cube-text-logo.png)
Cube is a four-elemental, fire-sampled virtual execution environment for Bitcoin.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. Clone the repository and navigate into the project directory:

```sh
git clone https://github.com/cube-btc/cube
cd cube
```

## Usage

Run the program with the following command:

```sh
cargo run <mode> <chain> <kind> <bitcoin-rpc-url> <bitcoin-rpc-user> <bitcoin-rpc-password>
```

### Parameters:

- `<mode>`: Whether to run in pruned or archival mode. Supported values:
  - `pruned`: For running in pruned mode.
  - `archival`: For running in archival node.
- `<chain>`: The Bitcoin network to use. Supported values:
  - `signet`
  - `mainnet`
- `<kind>`: The kind of running mode. Supported values:
  - `node`: For running a Cube node.
  - `engine`: For network operators.
- `<bitcoin-rpc-url>`: The RPC URL of the Bitcoin node.
- `<bitcoin-rpc-user>`: The RPC username of the Bitcoin node.
- `<bitcoin-rpc-password>`: The RPC password of the Bitcoin node.

### Example:

```sh
cargo run pruned signet node http://127.0.0.1:38332 user password
```

## License

This project is licensed under the CC0 1.0 Universal License. See the `LICENSE` file for details.
