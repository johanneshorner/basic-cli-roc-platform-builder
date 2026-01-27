A port of [basic-cli](https://github.com/roc-lang/basic-cli/tree/migrate-zig-compiler) using [roc-platform-builder](https://github.com/johanneshorner/roc-platform-builder).

```
nix develop
just build-musl
echo "bla" | roc --no-cache examples/main.roc -- /tmp/roc-test.txt
```
