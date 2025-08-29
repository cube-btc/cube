![Cube](https://i.ibb.co/KjnGsD7L/cube-text-logo.png)
Cube is a virtual machine designed to enable bridgeless smart contract execution natively on Bitcoin.

> Cube is currently in the early development phase.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. Clone the repository and navigate into the project directory:

```sh
git clone https://github.com/cube-vm/cube
cd cube
```

## Usage

Run the program with the following command:

```sh
cargo run <chain> <mode> <bitcoin-rpc-url> <bitcoin-rpc-user> <bitcoin-rpc-password>
```

### Parameters:

- `<chain>`: The Bitcoin network to use. Supported values:
  - `signet`
  - `mainnet`
- `<mode>`: The mode in which the program runs. Supported values:
  - `node`: For running a Cube node.
  - `engine`: For network operators.
- `<bitcoin-rpc-url>`: The RPC URL of the Bitcoin node.
- `<bitcoin-rpc-user>`: The RPC username.
- `<bitcoin-rpc-password>`: The RPC password.

### Example:

```sh
cargo run signet node http://127.0.0.1:38332 user password
```

## Contributing

We welcome contributions! Please [check the areas where we need help](https://github.com/cube-vm/cube/blob/main/CONTRIB.md) for more details on how you can contribute.

## License

This project is licensed under the CC0 1.0 Universal License. See the `LICENSE` file for details.
