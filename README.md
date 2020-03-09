# ipld

README and repos under construction

## Install

**NOTE: due to our use of `#![feature(specialization)]` in order to re-use [`serde`](https://serde.rs) for IPLD codecs, you'll need to use the nightly compiler**

```toml
[dependencies]
ipld = "0.0.3"

[features]
dag-cbor = "uses serde_cbor to implement the `DagCbor` format"
dag-json = "uses serde_json to implement the `DagJson` format"
multicodec = "enables all listed IPLD formats"
```

## License

MIT or Apache-2.0.

Originally forked from [`rust-ipld`](https://github.com/rust-ipfs/rust-ipld) to
integrate `serde` and implement IPLD Schemas and Representations.
