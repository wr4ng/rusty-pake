# rusty-pake
**Course**: [02232 Applied Cryptography](https://kurser.dtu.dk/course/02232)

Rust toy implementation of the SPAKE2+ protocol using [`curve25519-dalek`](https://crates.io/crates/curve25519-dalek) as the prime order finite abelian group.

# Compilation and Installation
The following setup steps have been tested on a Ubunto 24.04.3 LTS (slightly newer than the mentioned version).

1. Install fresh Ubuntu VM
2. Update packages and install project dependencies:
```shell
sudo apt update && sudo apt upgrade -y
sudo apt install git curl build-essential libssl-dev pkg-config -y
```
3. Install `rust` as per the [installation instructions](https://rust-lang.org/learn/get-started/):
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
The shell needs to be restarted to make `cargo` available.

4. Clone the project:
```shell
git clone https://github.com/wr4ng/rusty-pake.git
cd rusty-pake
```
Then the setup is complete and the environment is ready to run the application and test cases.

# Running the test cases
Run automated test-suite:
```shell
cargo test
```
This runs the internal unit tests but also end-to-end tests where the server is spun up for each of the tests.

To run the server and client manually, use the following commands (in separate terminals)
```shell
cargo run --bin=server
cargo run --bin=client
```

When running the client locally no URL need to be entered (the default can be used).
Then enter `setup` and afterwards an id and password.
Using the same id and password in `exchange` yields a key that is the same the server computes.
Can be verified using `verify`.

The server id can be controlled using the `SERVER_ID` environment variable,
and the port using the `PORT` environment variable:
```shell
SERVER_ID=some-other-id PORT=4242 cargo run --bin=server
```

We also provide the following binaries as exmaple clients that run against the local server using predefined options. These require the server to be running locally in a separate process. These expect the default port (3000) and server id (SPAKE2+).

```shell
cargo run --bin=example-client        # A single client setup, exchange and verification
cargo run --bin=example-many-clients  # 4 clients performing setup and then exchange + verify 20 times each
```
