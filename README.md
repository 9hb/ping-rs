# ping-rs

Simple ping utility written in Rust.

## Usage

```bash
ping-rs <target_addr> [OPTIONS]
```

### Options

- `-c, --count <amount>` - Number of ping packets (default: 4)
- `-t, --timeout <seconds>` - Timeout in seconds (default: 5)
- `-s, --size <bytes>` - Data size in bytes (default: 32)
- `-i, --interval <ms>` - Interval between pings in milliseconds (default: 1000)

### Examples

```bash
ping-rs google.com
ping-rs 8.8.8.8 -c 10
ping-rs example.com -t 3 -s 64
```

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run -- google.com
```

**Note:** Raw sockets require elevated privileges on most systems.
