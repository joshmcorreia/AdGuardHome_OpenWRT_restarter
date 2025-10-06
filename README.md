# Build Pre-requisites
[cross-rs](https://github.com/cross-rs/cross)
```
$ cargo install cross --git https://github.com/cross-rs/cross
```

# Building the binary
```
$ make build
```

# Copying the binary to your router
```
$ scp target/aarch64-unknown-linux-musl/release/adguardhome_openwrt_restarter ROUTER:/root
```
