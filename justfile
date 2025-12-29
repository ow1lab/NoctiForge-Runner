# Default command: List all available just commands
default:
    @just --list

test:
    @cargo test

lint:
    @cargo clippy

update:
    nix flake update
    cargo update
