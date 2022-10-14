# validator setup

Core scripts come from [Edgevana](https://github.com/shiraz-edgevana/solana).

## Initial Setup
1. Setup core server
2. Install rust toolchain
3. Build geyser-plugin
4. Install nats-server
5. Create nats-server service
6. Build indexer
7. Create indexer service
8. Create script to launch validator with plugin

Need to install build-essentials pkg-config libssl-dev

## Update
1. Pull `bokoup-program-library` and `geyser-plugin-nats` repos
2. `cargo-build --release` in each
3. `cp /home/ubuntu/bokoup-program-library/target/release/libbpl_indexer.so /usr/local/bin`
4. `cp /home/ubuntu/geyser-plugin-nats/target/release/libgeyser_plugin_nats.so /home/sol`
5. `cp /home/ubuntu/geyser-plugin-nats/config.json /home/sol`
6. Update `config.json` to point to the correct location for `libgeyser_plugin_nats.so`