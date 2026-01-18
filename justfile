default:
    just --list

build-host:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --target x86_64-unknown-linux-musl

    mkdir -p platform/targets/x64musl
    cp target/x86_64-unknown-linux-musl/debug/libhost.a platform/targets/x64musl/
    cp -f $(musl-gcc -print-file-name=crt1.o) platform/targets/x64musl/
    cp -f $(musl-gcc -print-file-name=libc.a) platform/targets/x64musl/
