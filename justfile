set windows-powershell := true

test:
    cargo test

clean:
    cargo clean

check:
    cargo check

simple:
    cargo run --package simple

adaptor:
    cargo run --package adaptor
