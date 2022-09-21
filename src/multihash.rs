use crate::dev::*;
use multihash::Hasher;

macro_rules! impl_multihasher {
    (@multihash $(
        $(#[$meta:meta])*
        $variant:ident ($const:ident: $code:expr) -> $ty:ty [$size:expr],
    )*) => {
        /// A generic [multihash]()-er enum.
        #[derive(Debug)]
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
            pub const fn code(&self) -> u64 {
                match self {
                    $(Self::$variant(_) => $code,)*
                }
            }

            ///
            #[inline]
            pub const fn size(&self) -> u8 {
                match self {
                    $(Self::$variant(_) => $size,)*
                }
            }

            ///
            pub fn finalize(&mut self) -> Result<DefaultMultihash, Error> {
                let mh = DefaultMultihash::wrap(self.code(), multihash::Hasher::finalize(self))?;
                self.reset();
                Ok(mh)
            }
        }

        impl multihash::Hasher for Multihash {
            fn update(&mut self, input: &[u8]) {
                match self {
                    $(Self::$variant(hasher) => hasher.update(input),)*
                }
            }
            fn finalize(&mut self) -> &[u8] {
                match self {
                    $(Self::$variant(hasher) => hasher.finalize(),)*
                }
            }
            fn reset(&mut self) {
                match self {
                    $(Self::$variant(hasher) => hasher.reset(),)*
                }
            }
        }

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
    Sha2_256 (SHA2_256: 0x12) -> multihash::Sha2_256 [32],
    ///
    Sha2_512 (SHA2_512: 0x13) -> multihash::Sha2_512 [64],
    ///
    Sha3_224 (SHA3_224: 0x17) -> multihash::Sha3_224 [28],
    ///
    Sha3_256 (SHA3_256: 0x16) -> multihash::Sha3_256 [32],
    ///
    Sha3_384 (SHA3_384: 0x15) -> multihash::Sha3_384 [48],
    ///
    Sha3_512 (SHA3_512: 0x14) -> multihash::Sha3_512 [64],
    ///
    Keccak224 (KECCAK_224: 0x1a) -> multihash::Keccak224 [28],
    ///
    Keccak256 (KECCAK_256: 0x1b) -> multihash::Keccak256 [32],
    ///
    Keccak384 (KECCAK_384: 0x1c) -> multihash::Keccak384 [48],
    ///
    Keccak512 (KECCAK_512: 0x1d) -> multihash::Keccak512 [64],
    ///
    Blake2b256 (BLAKE2B_256: 0xb220) -> multihash::Blake2bHasher::<32> [32],
    ///
    Blake2b512 (BLAKE2B_512: 0xb240) -> multihash::Blake2bHasher::<64> [64],
    ///
    Blake2s128 (BLAKE2S_128: 0xb250) -> multihash::Blake2sHasher::<16> [16],
    ///
    Blake2s256 (BLAKE2S_256: 0xb260) -> multihash::Blake2sHasher::<32> [32],
    ///
    Blake3_256 (BLAKE3_256: 0x1e) -> multihash::Blake3Hasher::<32> [32],
}
