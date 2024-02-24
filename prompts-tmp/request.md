
you can set binary name in Cargo.toml
```toml
[package]
description = "The fast, collaborative code editor."
edition = "2021"
name = "zed"
version = "0.125.0"
publish = false
license = "GPL-3.0-or-later"

[lib]
name = "zed"
path = "src/zed.rs"
doctest = false

[[bin]]
name = "Zed"
path = "src/main.rs"

[dependencies]
activity_indicator.workspace = true
ai.workspace = true
anyhow.workspace = true
assets.workspace = true
assistant.workspace = true
async-compression.workspace = true
async-recursion = "0.3"
```
In this case you will run it using:
```bash
cargo run --bin Zed
```