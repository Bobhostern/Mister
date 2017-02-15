#!/bin/bash
set -ev
cargo build --verbose
cargo test -p mister-core --verbose
