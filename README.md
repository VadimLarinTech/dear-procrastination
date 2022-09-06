# Near todo manager "Dear Procrastination"

## Description

"Dear Procrastination" it's a todo manager with financial motivation to complete tasks
This contract implements simple todo manager backed by storage on blockchain.
Contract in `src/lib.rs` provides methods to crate / get tasks and setting completed task status.

When creating a new task, the user must specify a deadline and make a deposit, which is returned if the task is completed on time.

Application deployment implemented via web4 (https://github.com/vgrichina/web4)

### Links:

https://manager4.testnet.page - testnet


## Installation

```
git clone https://github.com/vadimlarintech/dear-procrastination
```


## Setup
Install dependencies:

```
yarn
```

If you don't have `Rust` installed, complete the following 3 steps:

1) Install Rustup by running:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

([Taken from official installation guide](https://www.rust-lang.org/tools/install))

2) Configure your current shell by running:

```
source $HOME/.cargo/env
```

3) Add wasm target to your toolchain by running:

```
rustup target add wasm32-unknown-unknown
```

Next, make sure you have `near-cli` by running:

```
near --version
```

If you need to install `near-cli`:

```
npm install near-cli -g
```

## Login
If you do not have a NEAR account, please create one with [NEAR Wallet](https://wallet.testnet.near.org).

In the project root, login with `near-cli` by following the instructions after this command:

```
near login
```

Deploy your example!

```
near deploy your-account.tesnet --wasmFile ./res/your-project.wasm
```

## To Test

```
yarn test
```

## To Explore

- `src/lib.rs` for the contract code
- `res/` for the front-end HTML
- `tests/integration/main.rs` for integration tests

## To Build the Documentation

```
cargo doc --no-deps --open
```


## Future Development

Some ideas for future feature development:

- Improvement of the UI/UX
- Sending user deposits to farming for efficient use of funds and payment of income to users
- Rewarding NFT users who have achieved significant success in managing their tasks

## Key Contributors

- [Vadim Larin - @vadimlarintech](https://github.com/vadimlarintech)