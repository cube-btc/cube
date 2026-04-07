![Cube](https://i.ibb.co/KjnGsD7L/cube-text-logo.png)
Cube is a four-elemental, fire-sampled virtual execution environment for Bitcoin.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. Clone the repository and navigate into the project directory:

```sh
git clone https://github.com/cube-btc/cube
cd cube
```

## Generating a Secret Key

If you don't already have a secret key, you can optionally generate a new one by running:

```sh
cargo run gensec
```

This will generate a random secret key and print it as an nsec string, which you'll need when running the program.

Cube abides by the [NIP-19](https://nips.nostr.com/19) format for secret keys, which uses bech32-encoded `nsec` strings for private keys.

## Usage

Run the program with the following command:

```sh
cargo run <resource-mode> <chain> <kind> <rpc-url> <rpc-user> rpc-password> <syncinflight?>
```

### Parameters:

- `<resource-mode>`: Whether to run in pruned or archival mode. Supported values:
  - `pruned`: For running in pruned mode.
  - `archival`: For running in archival mode.
- `<chain>`: The Bitcoin network to use. Supported values:
  - `signet`
  - `mainnet`
- `<kind>`: The kind of operating entity. Supported values:
  - `node`: For running a Cube node.
  - `engine`: For the network operator.
- `<rpc-url>`: The RPC URL of the Bitcoin node.
- `<rpc-user>`: The RPC username of the Bitcoin node.
- `<rpc-password>`: The RPC password of the Bitcoin node.
- `<syncinflight?>`: Whether to sync in-flight unconfirmed executions.

### Example:

```sh
cargo run pruned signet node http://127.0.0.1:38332 user password true
```

## License

This project is licensed under the CC0 1.0 Universal License. See the `LICENSE` file for details.
