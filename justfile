default:
    just --list

build-musl:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --target x86_64-unknown-linux-musl

    mkdir -p platform/targets/x64musl
    cp target/x86_64-unknown-linux-musl/debug/libhost.a platform/targets/x64musl/
    cp -f $(musl-gcc -print-file-name=crt1.o) platform/targets/x64musl/
    cp -f $(musl-gcc -print-file-name=libc.a) platform/targets/x64musl/

build-glibc:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --target x86_64-unknown-linux-gnu

    mkdir -p platform/targets/x64glibc
    cp target/x86_64-unknown-linux-gnu/debug/libhost.a platform/targets/x64glibc/
    cp -f $(gcc -print-file-name=Scrt1.o) platform/targets/x64glibc/
    cp -f $(gcc -print-file-name=crti.o) platform/targets/x64glibc/
    cp -f $(gcc -print-file-name=libc.so) platform/targets/x64glibc/
    cp -f $(gcc -print-file-name=crtn.o) platform/targets/x64glibc/
    cp -f $(gcc -print-file-name=libgcc_s.so.1) platform/targets/x64glibc/
