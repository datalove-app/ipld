//! Execution contexts for `Representation`s to `Read`/`Write` themselves from/to bytes and query/mutate themselves by specializing their implementation around specific `State` changes.
//!
//! While a `Representation` defines how a type traverses it's fields and maps them to bytes or blocks, the `Context` determines what happens with the bytes when encountering nested types, links, etc, before writing to or after reading from the byte stream.
//!
//! For example:
//!     - An `impl Context for EncryptedContext` can provide a byte stream that encrypts bytes written by a type/decrypts bytes read into a type. Later, a `Representation` can be provided with an `EncyptedContext` initialized with a key, transparently encrypting/decrypting the provided byte streams.
//!     - Additionally, we can define an `impl State for Encrypted<R, W>: Context<R, W>` and a type whose `Representation` implementation could derive an encryption/decryption key from within the type, ensuring that the type can only be stored in ciphertext.

#[cfg(feature = "ipfs")]
mod ipfs;

use crate::dev::*;
use maybestd::{
    collections::HashMap,
    io::{empty, Cursor, Empty, Read, Sink, Write},
};

// trait BlockWriter: Write {}

/// Trait for providing blocks and additional logic required for selection,
/// patching, etc.
pub trait Context: Sized {
    ///
    type Reader: Read; // ? BufRead?

    ///
    type Writer: Write; // ? BufWrite?

    // type Marker;

    ///
    fn block_reader(&mut self, cid: &Cid) -> Result<Self::Reader, Error>;

    /// TODO:
    fn block_writer(&mut self, cid_to_replace: Option<&Cid>) -> Result<Self::Writer, Error> {
        unimplemented!()
    }

    // ///
    // fn flush_writer(&mut self, cid_to_replace: Option<&Cid>, block: ) -> Result<Cid, Error> {

    // }

    //
    // fn decoder<'de, 'a: 'de>(&mut self) -> Box<dyn ErasedDeserializer<'de> + 'a> {
    //     unimplemented!()
    // }
    //
    // fn encoder(&mut self) -> Box<dyn ErasedSerializer + '_> {
    //     unimplemented!()
    // }
    //
    // fn set_decoder<'de, D: Decoder<'de>>(&mut self, de: &mut D) {
    //     unimplemented!()
    // }
    //
    // fn block_encoder<Si>(
    //     &mut self,
    //     meta: BlockMeta<'_, Si>,
    // ) -> Result<&'_ mut dyn ErasedSerializer, Error>
    // where
    //     Si: MultihashSize,
    // {
    //     unimplemented!()
    // }
    //
    // fn path_encoder<P: AsRef<Path>>(
    //     &mut self,
    //     meta: P,
    // ) -> Result<&'_ mut dyn ErasedSerializer, Error> {
    //     unimplemented!()
    // }
    //
    // fn close_encoder<Si, So>(
    //     &mut self,
    //     replacing: Option<BlockMeta<'_, Si, So>>,
    // ) -> Result<CidGeneric<So>, Error>
    // where
    //     Si: MultihashSize,
    //     So: MultihashSize,
    // {
    //     unimplemented!()
    // }

    //
    //
    //
    // /// Internally, this will:
    // ///     - get a (concrete?) BlockWriter from a BlockService
    // ///     - determine the Codec + Format from the BlockMeta
    // ///         - create a
    // ///
    // /// ## Example:
    // /// ```
    // /// Context::write(&ipld).await?;
    // /// ```
    // async fn write<B, R>(&mut self, dag: &R, block_meta: B) -> Result<(), ()>
    // where
    //     R: Representation<Self>,
    //     B: Into<BlockMeta>;
    //
    // async fn resolve(&mut self);
}

impl<'a, C: Context + 'a> Context for &'a mut C {
    type Reader = C::Reader;
    type Writer = C::Writer;

    ///
    fn block_reader(&mut self, cid: &Cid) -> Result<Self::Reader, Error> {
        (*self).block_reader(cid)
    }
}

impl Context for () {
    type Reader = Empty;
    type Writer = Sink;

    fn block_reader(&mut self, _: &Cid) -> Result<Self::Reader, Error> {
        Err(Error::Context(anyhow::Error::msg("empty block")))
    }
}

///
#[derive(Clone, Debug, Default)]
pub struct MemoryContext {
    blocks: HashMap<Cid, Vec<u8>>,
}

impl MemoryContext {
    ///
    /// TODO? replace args with BlockMeta
    pub fn add_block(
        &mut self,
        version: Version,
        multicodec_code: u64,
        multihash_code: u64,
        block: Vec<u8>,
    ) -> Result<Cid, Error> {
        let cid = Cid::from_reader(version, multicodec_code, multihash_code, block.as_slice())?;
        self.blocks.insert(cid, block);
        Ok(cid)
    }
}

impl Context for MemoryContext {
    type Reader = Cursor<Vec<u8>>;
    type Writer = Vec<u8>;

    fn block_reader(&mut self, cid: &Cid) -> Result<Self::Reader, Error> {
        let block = self
            .blocks
            .get(cid)
            .ok_or_else(|| Error::Context(anyhow::anyhow!("missing block for cid: {:?}", cid)))?;
        Ok(Cursor::new(block.to_owned()))
    }
}
