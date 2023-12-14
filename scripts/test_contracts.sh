#!/bin/bash

# Run cargo init
cargo run init

# Define an array of contract paths
contract_paths=(
    "contracts/PSP22"
    "contracts/PSP22/extensions/tests/burnable"
    "contracts/PSP22/extensions/tests/mintable"
    "contracts/PSP22/extensions/tests/capped"
    "contracts/PSP22/extensions/tests/pausable"
    "contracts/PSP22/extensions/tests/wrapper"
)

# Loop through each path and run cargo test
for path in "${contract_paths[@]}"; do
    (
        printf "\n  \033[44m\033[30m  Testing in %s  \033[0m\n\n" "$path"
        cd "$path" && cargo test
    )
done
