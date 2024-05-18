# Pontus Onyx

A [remoteStorage](https://remotestorage.io/) server and client.

```txt
⚠ Warning : it is not production-ready until version 1.0.0 !
So 0.x.0 versions are breaking changes until then.
```

Based on [IETF Draft of the 19 December 2023](https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-22).

This workspace contains 3 crates :

- `pontus_onyx`
    - with `client` feature, this is a client library (to use with webassembly)
    - with `server` feature, this is a server library (for embeddable projects in Rust)
- `pontus_onyx_cli`
    - this is a command-line server binary (to use the server library directly)
- `pontus_onyx_gui`
    - this is a graphical server binary (to use the server library directly)
