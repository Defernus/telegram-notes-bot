set export
set dotenv-load

default:
    @just --list

run:
    cargo run --bin bot
    