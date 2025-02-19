# keneki

keneki is a simple utility for quarantining a program; a Faraday's Cage for your external software.

## how it works

keneki builds around the program `unshare`, distributed with most linux distributions.
it is a very simple program that wraps another at runtime.

```bash
unshare -r -n curl example.com

curl: (6) Could not resolve host: example.com
```

my use case however requires a static binary; such that if you're given that binary,
you'll have a hard time recovering the original non-quarantined binary.

## usage

```bash
EMBEDDED_BINARY=$(which curl) cargo build --release
./target/release/keneki example.com

# curl: (6) Could not resolve host: example.com

rm -rf target
EMBEDDED_BINARY=$(which python3) cargo build --release
./target/release/keneki -c 'import requests; requests.get("https://example.com")'

# [Errno -3] Temporary failure in name resolution
```
