# ipld

README and repos under construction

## Install

**NOTE: due to our use of `#![feature(specialization)]` to leverage existing [`serde`](https://serde.rs) codecs (particularly for DAGs), you'll need to use the nightly compiler**

```toml
[dependencies]
ipld = { git = "https://github.com/datalove-app/ipld" }
```

## License

MIT or Apache-2.0.

Originally forked from [`rust-ipld`](https://github.com/rust-ipfs/rust-ipld).
