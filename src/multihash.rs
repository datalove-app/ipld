use crate::dev::*;
use macros::derive_more::From;
use multihash::Hasher;

macro_rules! impl_multihasher {
    (@multihash $(
        $(#[$meta:meta])*
        $variant:ident ($const:ident: $name:literal: $code:expr) -> $ty:ty [$size:expr],
    )*) => {
        /// A generic [multihash]()-er enum.
        #[derive(Debug, From)]
        #[non_exhaustive]
        pub enum Multihash {
            $(
                $(#[$meta])*
                $variant($ty),
            )*
        }

        impl Multihash {
            $(
                ///
                pub const $const: u64 = $code;
            )*

            ///
            #[inline]
            pub fn from_code<const C: u64>() -> Result<Self, Error> {
                Ok(match C {
                    $($code => Ok(Self::$variant(<$ty>::default())),)*
                    code => Err(multihash::Error::UnsupportedCode(code))
                }?)
            }

            ///
            #[inline]
            pub const fn is_supported<const C: u64>() -> bool {
                match C {
                    $($code => true,)*
                    _ => false,
                }
            }

            ///
            #[inline]
            pub const fn code(&self) -> u64 {
                match self {
                    $(Self::$variant(_) => $code,)*
                }
            }

            ///
            #[inline]
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => $name,)*
                }
            }

            /// The underlying size of the generated digest.
            #[inline]
            pub const fn size(&self) -> u8 {
                match self {
                    $(Self::$variant(_) => $size,)*
                }
            }

            ///
            pub fn try_finalize(&mut self) -> Result<DefaultMultihash, Error> {
                let mh = DefaultMultihash::wrap(self.code(), multihash::Hasher::finalize(self))?;
                self.reset();
                Ok(mh)
            }
        }

        impl multihash::Hasher for Multihash {
            #[inline]
            fn update(&mut self, input: &[u8]) {
                match self {
                    $(Self::$variant(hasher) => hasher.update(input),)*
                }
            }
            #[inline]
            fn finalize(&mut self) -> &[u8] {
                match self {
                    $(Self::$variant(hasher) => hasher.finalize(),)*
                }
            }
            #[inline]
            fn reset(&mut self) {
                match self {
                    $(Self::$variant(hasher) => hasher.reset(),)*
                }
            }
        }

        impl std::io::Write for Multihash {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.update(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        // #[cfg(feature = "digest")]
        // mod _digest {
        //     use super::*;
        //     use digest::{Digest, OutputSizeUser, FixedOutputReset, generic_array::{typenum::U64, GenericArray}};
        //
        //     impl OutputSizeUser for Multihash {
        //         type OutputSize = GenericArray<u8, U64>;
        //     }
        //
        //     impl Multihash {
        //         const OUTPUT_SIZE: usize = U64::USIZE;
        //
        //         fn finalize_to_digest(&mut self, digest: &mut Output<Self>) {
        //             digest.as_mut_slice()
        //         }
        //     }
        //
        //     impl Digest for Multihash {
        //         fn new() -> Self {
        //             unimplemented!()
        //         }
        //         fn new_with_prefix(data: impl AsRef<[u8]>) -> Self {
        //             let mut new = Self::new();
        //             new.update(data);
        //             new
        //         }
        //         fn update(&mut self, data: impl AsRef<[u8]>) {
        //             multihash::Hasher::update(self, data.as_ref())
        //         }
        //         fn chain_update(mut self, data: impl AsRef<[u8]>) -> Self {
        //             (&mut self).update(data);
        //             self
        //         }
        //         fn finalize(mut self) -> Output<Self> {
        //             let mut slice = Output::<Self>::default();
        //             let bytes = multihash::Hasher::finalize(&mut self);
        //             Ok(From::from())
        //         }
        //         fn finalize_into(mut self, out: &mut Output<Self>) {
        //             unimplemented!()
        //         }
        //         fn finalize_reset(&mut self) -> Output<Self>
        //         where
        //             Self: FixedOutputReset,
        //         {
        //             unimplemented!()
        //         }
        //         fn finalize_into_reset(&mut self, out: &mut Output<Self>)
        //         where
        //             Self: FixedOutputReset,
        //         {
        //             unimplemented!()
        //         }
        //         fn reset(&mut self)
        //         where
        //             Self: Reset,
        //         {
        //             multihash::Hasher::reset(self)
        //         }
        //         fn output_size() -> usize {
        //             Self::OUTPUT_SIZE
        //         }
        //         fn digest(data: impl AsRef<[u8]>) -> Output<Self> {
        //             let mut mh = Self::new();
        //             mh.update(data);
        //             mh.finalize()
        //         }
        //     }
        // };

        impl TryFrom<u64> for Multihash {
            type Error = Error;
            fn try_from(multihash_code: u64) -> Result<Self, Self::Error> {
                Ok(match multihash_code {
                    $($code => Ok(Self::$variant(<$ty>::default())),)*
                    mh_code => Err(multihash::Error::UnsupportedCode(mh_code))
                }?)
            }
        }
    };
}

impl_multihasher! {@multihash
    ///
    Sha2_256 (SHA2_256: "sha2-256": 0x12)
        -> multihash::Sha2_256 [32],
    ///
    Sha2_512 (SHA2_512: "sha2-512": 0x13)
        -> multihash::Sha2_512 [64],
    ///
    Sha3_224 (SHA3_224: "sha3-224": 0x17)
        -> multihash::Sha3_224 [28],
    ///
    Sha3_256 (SHA3_256: "sha3-256": 0x16)
        -> multihash::Sha3_256 [32],
    ///
    Sha3_384 (SHA3_384: "sha3-384": 0x15)
        -> multihash::Sha3_384 [48],
    ///
    Sha3_512 (SHA3_512: "sha3-512": 0x14)
        -> multihash::Sha3_512 [64],
    ///
    Keccak224 (KECCAK_224: "keccak-224": 0x1a)
        -> multihash::Keccak224 [28],
    ///
    Keccak256 (KECCAK_256: "keccak-256": 0x1b)
        -> multihash::Keccak256 [32],
    ///
    Keccak384 (KECCAK_384: "keccak-384": 0x1c)
        -> multihash::Keccak384 [48],
    ///
    Keccak512 (KECCAK_512: "keccak-512": 0x1d)
        -> multihash::Keccak512 [64],
    ///
    Blake2b256 (BLAKE2B_256: "blake2b-256": 0xb220)
        -> multihash::Blake2bHasher::<32> [32],
    ///
    Blake2b512 (BLAKE2B_512: "blake2b-512": 0xb240)
        -> multihash::Blake2bHasher::<64> [64],
    ///
    Blake2s128 (BLAKE2S_128: "blake2s-128": 0xb250)
        -> multihash::Blake2sHasher::<16> [16],
    ///
    Blake2s256 (BLAKE2S_256: "blake2s-256": 0xb260)
        -> multihash::Blake2sHasher::<32> [32],
    ///
    Blake3_256 (BLAKE3_256: "blake3": 0x1e)
        -> multihash::Blake3Hasher::<32> [32],
}
