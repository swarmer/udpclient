#!/usr/bin/env bash
set -e
cargo +nightly fmt
cmds/check.sh
cargo test
