# geyser-indexer


# Usage

1. MAKE SURE YOU HAVE SOLANA VERSION 1.9.15 INSTALLED 

```
solana --version
solana-cli 1.9.15 (src:e0254200; feat:1070292356)
```

2. `cargo build`
3. check your `target/debug` folder - depending on your platform you should have either a `.dylib` or `.so` file
4. make sure that the `Config.json` file points to the correct library (may have to switch to `.so`)