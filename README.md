# Rust Project Template


## getting started

```
git clone https://github.com/ItoYo16u/rust-template <dirname>
cd <dirname>
```

### Install toolchain

```bash
cargo check 
# or cargo build

```

### run

```bash
# build and run
cargo run
# clean
cargo clean
```

#### run in specific channel
```shell
rustup run nightly cargo <command>
```

### fmt

```bash

rustup component add rustfmt
cargo fmt
```

### lint

```bash
rustup component add clippy
```
