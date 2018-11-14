#!/bin/sh

if [[ -e /run/aeterno/sys.sock ]]
then rm /run/aeterno/sys.sock
fi

if [[ -e /run/aeterno/master.sock ]]
then rm /run/aeterno/master.sock
fi

cargo build && RUST_LOG=debug cargo run --bin aeterno-init
