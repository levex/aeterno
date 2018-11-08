#!/bin/sh

if [[ -e /run/aeterno/sys.sock ]]
then rm /run/aeterno/sys.sock
fi

cargo build && RUST_LOG=debug cargo run --bin aeterno-init
