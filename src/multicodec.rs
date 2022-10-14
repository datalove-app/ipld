use crate::dev::*;
use maybestd::{
    convert::TryFrom,
    io::{Read, Write},
};

pub use multicodec::Multicodec;

/// An unified trait for all IPLD
/// [Codec](https://github.com/ipld/specs/blob/master/block-layer/codecs/README.dsmd)s,
/// providing methods for reading and writing blocks.
pub trait Codec: TryFrom<u64, Error = Error> {
    /// The standardized [`Multicodec`] name for this IPLD codec.
    const NAME: &'static str;

    /// The standardized [`Multicodec`] code that identifies this IPLD codec.
    const CODE: u64;

    /// Given a dag, serialize it to a `Vec<u8>`.
    fn encode<T>(&mut self, dag: &T) -> Result<Vec<u8>, Error>
    where
        T: Representation,
    {
        let mut vec = vec![];
        self.write(dag, &mut vec)?;
        Ok(vec)
    }

    /// Given a dag and a `Write`, encode it to the writer.
    fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
    where
        T: Representation,
        W: Write;

    /// Given some bytes, deserialize a dag.
    fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Representation,
    {
        self.read::<T, _>(bytes)
    }

    /// Given a `Read`, deserialize a dag.
    fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
    where
        T: Representation,
        R: Read;
}

mod multicodec {
    use super::*;

    macro_rules! impl_multicodec {
    ($(
        #[cfg(feature = $feature:expr)]
        $(#[$meta:meta])*
        $variant:ident -> $ty:ty,
    )*) => {
        /// A generic [multicodec]() enum.
        #[derive(Clone, Debug)]
        #[non_exhaustive]
        pub enum Multicodec {
            $(
                #[cfg(feature = $feature)]
                $(#[$meta])*
                $variant($ty),
            )*
        }

        impl Multicodec {
            #[inline]
            pub const fn is_known<const C: u64>() -> bool {
                match C {
                    $(
                        #[cfg(feature = $feature)]
                        <$ty>::CODE => true,
                    )*
                    _ => false,
                }
            }

            /// The standardized name of the given codec.
            #[inline]
            pub const fn name(&self) -> &'static str {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(_) => <$ty>::NAME,
                    )*
                    // _ => unimplemented!()
                }
            }

            ///
            #[inline]
            pub const fn code(&self) -> u64 {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(_) => <$ty>::CODE,
                    )*
                    // _ => unimplemented!()
                }
            }

            ///
            #[inline]
            pub fn from_name(name: &str) -> Result<Self, Error> {
                match name {
                    $(
                        #[cfg(feature = $feature)]
                        <$ty>::NAME => Ok(Self::$variant(<$ty>::new())),
                    )*
                    name => Err(Error::UnknownMulticodecName(name.to_string()))
                }
            }

            ///
            #[inline]
            pub const fn from_code<const C: u64>() -> Result<Self, Error> {
                match C {
                    $(
                        #[cfg(feature = $feature)]
                        <$ty>::CODE => Ok(Self::$variant(<$ty>::new())),
                    )*
                    code => Err(Error::UnknownMulticodecCode(code))
                }
            }

            ///
            pub fn encode<T>(&mut self, dag: &T) -> Result<Vec<u8>, Error>
            where
                T: Representation,
            {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(inner) => inner.encode(dag),
                    )*
                    // _ => unimplemented!()
                }
            }

            ///
            pub fn write<T, W>(&mut self, dag: &T, writer: W) -> Result<(), Error>
            where
                T: Representation,
                W: Write,
            {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(inner) => inner.write(dag, writer),
                    )*
                    // _ => unimplemented!()
                }
            }

            ///
            pub fn decode<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
            where
                T: Representation,
            {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(inner) => inner.decode(bytes),
                    )*
                    // _ => unimplemented!()
                }
            }

            ///
            pub fn read<T, R>(&mut self, reader: R) -> Result<T, Error>
            where
                T: Representation,
                R: Read,
            {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(inner) => inner.read(reader),
                    )*
                    // _ => unimplemented!()
                }
            }

            #[doc(hidden)]
            pub fn read_with_seed<Ctx, T, R>(
                &mut self,
                seed: SelectorSeed<'_, Ctx, T>,
                reader: R,
            ) -> Result<(), Error>
            where
                Ctx: Context,
                T: Select<Ctx>,
                R: Read,
            {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$variant(_) => <$ty>::read_with_seed(seed, reader),
                    )*
                }
            }

            #[inline]
            #[doc(hidden)]
            pub fn serialize_bytes<const C: u64, S>(
                dag: impl AsRef<[u8]>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::serialize_bytes(dag, serializer);
                }
                serializer.serialize_bytes(dag.as_ref())
            }

            #[inline]
            #[doc(hidden)]
            pub fn serialize_link<const C: u64, S>(
                cid: &Cid,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::serialize_link(cid, serializer);
                }
                #[cfg(feature = "dag-cbor")]
                if C == DagCbor::CODE {
                    return DagCbor::serialize_link(cid, serializer);
                }
                Serialize::serialize(&cid.inner, serializer)
            }

            #[inline]
            #[doc(hidden)]
            pub fn deserialize_any<'de, const C: u64, D, V>(
                deserializer: D,
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                D: Deserializer<'de>,
                V: LinkVisitor<'de>,
            {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::deserialize_any(deserializer, visitor);
                }
                #[cfg(feature = "dag-cbor")]
                if C == DagCbor::CODE {
                    return DagCbor::deserialize_any(deserializer, visitor);
                }
                deserializer.deserialize_any(visitor)
            }

            #[inline]
            #[doc(hidden)]
            pub fn deserialize_bytes<'de, const C: u64, D, V>(
                deserializer: D,
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                D: Deserializer<'de>,
                V: LinkVisitor<'de>,
            {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::deserialize_bytes(deserializer, visitor);
                }
                deserializer.deserialize_bytes(visitor)
            }

            #[inline]
            #[doc(hidden)]
            pub fn deserialize_link<'de, const C: u64, D, V>(
                deserializer: D,
                visitor: V,
            ) -> Result<V::Value, D::Error>
            where
                D: Deserializer<'de>,
                V: LinkVisitor<'de>,
            {
                #[cfg(feature = "dag-json")]
                if C == DagJson::CODE {
                    return DagJson::deserialize_link(deserializer, visitor);
                }
                #[cfg(feature = "dag-cbor")]
                if C == DagCbor::CODE {
                    return DagCbor::deserialize_link(deserializer, visitor);
                }
                deserializer.deserialize_any(visitor)
            }
        }

        impl TryFrom<u64> for Multicodec {
            type Error = Error;
            #[inline]
            fn try_from(code: u64) -> Result<Self, Self::Error> {
                match code {
                    $(
                        #[cfg(feature = $feature)]
                        <$ty>::CODE => Ok(Self::$variant(<$ty>::new())),
                    )*
                    _ => Err(Error::UnknownMulticodecCode(code)),
                }
            }
        }

        impl<'a> TryFrom<&'a str> for Multicodec {
            type Error = Error;
            #[inline]
            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                match s {
                    $(
                        #[cfg(feature = $feature)]
                        <$ty>::NAME => Ok(Self::$variant(<$ty>::new())),
                    )*
                    _ => Err(Error::UnknownMulticodecName(s.into())),
                }
            }
        }
    }}

    impl_multicodec! {
        #[cfg(feature = "dag-cbor")]
        ///
        DagCbor -> DagCbor,
        #[cfg(feature = "dag-json")]
        ///
        DagJson -> DagJson,
        // Custom(Box<dyn Codec>),
    }

    impl<'a, const S: usize> TryFrom<&'a CidGeneric<S>> for Multicodec {
        type Error = Error;
        fn try_from(cid: &CidGeneric<S>) -> Result<Self, Self::Error> {
            Multicodec::try_from(cid.codec())
        }
    }

    impl<'a> TryFrom<&'a Cid> for Multicodec {
        type Error = Error;
        fn try_from(cid: &Cid) -> Result<Self, Self::Error> {
            Multicodec::try_from(cid.multicodec_code())
        }
    }
}

#[cfg(feature = "skipped")]
#[cfg(test)]
mod autoref {
    use super::*;

    // autoref-based specialization courtesy of
    // https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html

    #[derive(Debug, Default)]
    struct TestCid(Cid);
    impl Serialize for TestCid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            (&mut &mut &mut Encoder(serializer)).serialize_link(&self.0)
        }
    }

    struct Encoder<S: Serializer>(S);

    trait ViaGeneric {
        type Ok;
        type Error;
        fn serialize_link(&mut self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }
    // impl<'a, T> ViaGeneric for Encoder<T>
    // where
    //     T: Serializer,
    // {
    //     type Ok = <T as Serializer>::Ok;
    //     type Error = <T as Serializer>::Error;
    // impl<'a, S> ViaGeneric for &'a mut Encoder<S>
    // where
    //     S: Serializer,
    // {
    //     type Ok = <S as Serializer>::Ok;
    //     type Error = <S as Serializer>::Error;
    // impl<'a, T> ViaGeneric for &'a mut Encoder<&mut T>
    // where
    //     for<'b> &'b mut T: Serializer,
    //     // &'a mut T: Serializer,
    // {
    //     type Ok = <&'a mut T as Serializer>::Ok;
    //     type Error = <&'a mut T as Serializer>::Error;
    //     fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
    //         // self.0.serialize_bytes(cid.to_bytes().as_slice())
    //         Err(<Self::Error as ser::Error>::custom("use autoref"))
    //     }
    // }

    impl<'a, T> ViaGeneric for &'a mut Encoder<T>
    where
        T: Serializer,
    {
        type Ok = T::Ok;
        type Error = T::Error;
        fn serialize_link(&mut self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            // (&mut self.0).serialize_bytes(cid.to_bytes().as_slice())
            Err(<Self::Error as ser::Error>::custom("use autoref"))
        }
    }

    #[test]
    fn test_cid_generic() {
        println!("cid bytes: {:?}", Cid::default().to_bytes().as_slice());

        let mut bytes = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap_err();
        TestCid::default().serialize(&mut ser).unwrap_err();
        println!("cbor bytes: {:?}", &bytes);

        assert!(false)
    }

    trait Tag<const C: usize> {}

    trait ViaDagJson<W> {
        type Ok;
        type Error;
        fn serialize_link(&mut self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }
    impl<'a, T, W> ViaDagJson<W> for &'a mut &'a mut Encoder<T>
    where
        T: Serializer + Tag<{ 1 }>,
        W: std::io::Write,
    {
        type Ok = T::Ok;
        type Error = T::Error;
        // impl<'a, W: std::io::Write> ViaDagJson for &'a mut &'a mut Encoder<&mut serde_json::Serializer<W>> {
        //     type Ok = ();
        //     type Error = serde_json::Error;
        fn serialize_link(&mut self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            let cid_str = cid
                .to_string()
                .map_err(<Self::Error as ser::Error>::custom)?;
            (&mut self.0).serialize_newtype_variant("", 0, DagJson::SPECIAL_KEY, &cid_str)
        }
    }

    #[test]
    fn test_cid_dag_json() {
        let mut bytes = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap();
        TestCid::default().serialize(&mut ser).unwrap();
        println!("json str: {}", std::str::from_utf8(&bytes).unwrap());

        assert!(false)
    }
}

#[cfg(feature = "skipped")]
mod tagged {
    use super::*;
    use erased_serde;

    // autoref-based specialization courtesy of
    // https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html

    // #[derive(Debug, Default)]
    // struct TestCid(Cid);
    // impl Serialize for TestCid {
    //     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //     where
    //         S: Serializer,
    //     {
    //         // let serializer = Box::new(<dyn erased_serde::Serializer>::erase(serializer));
    //         // (&mut &mut Encoder(&mut serializer))
    //         // .serialize_link::<S>(&self.0)
    //         // .map_err(S::Error::custom)
    //
    //         // let ok = (&mut &mut Encoder(serializer))
    //         //     .serialize_link(&self.0)
    //         // Ok(unsafe { ok.take::<S::Ok>() })
    //         unimplemented!()
    //     }
    // }

    #[derive(Debug, Default)]
    struct TestCid(Cid);
    impl Serialize for TestCid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            (&mut &mut &mut Encoder(serializer)).serialize_link(&self.0)
        }
    }

    trait Tag<const C: usize> {}

    struct Encoder<S>(S);

    trait ViaGeneric {
        type Ok;
        type Error;

        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }
    impl<'a, T> ViaGeneric for &'a mut Encoder<T>
    where
        T: Serializer,
    {
        type Ok = <T as Serializer>::Ok;
        type Error = <T as Serializer>::Error;
        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            // self.0.serialize_bytes(cid.to_bytes().as_slice())
            Err(<Self::Error as ser::Error>::custom("use autoref"))
        }
    }

    #[test]
    fn test_cid_generic() {
        println!("cid bytes: {:?}", Cid::default().to_bytes().as_slice());

        let mut bytes = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap_err();
        TestCid::default().serialize(&mut ser).unwrap_err();
        println!("cbor bytes: {:?}", &bytes);

        assert!(false)
    }

    trait ViaDagJson {
        type Ok;
        type Error;
        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error>;
    }
    impl<'a, T> ViaDagJson for &'a mut &'a mut Encoder<T>
    where
        T: Serializer + Tag<{ 1 }>,
    {
        type Ok = T::Ok;
        type Error = T::Error;
        fn serialize_link(self, cid: &Cid) -> Result<Self::Ok, Self::Error> {
            let cid_str = cid
                .to_string()
                .map_err(<Self::Error as ser::Error>::custom)?;
            self.0
                .serialize_newtype_variant("", 0, DagJson::SPECIAL_KEY, &cid_str)
        }
    }

    #[test]
    fn test_cid_dag_json() {
        let mut bytes = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap();
        TestCid::default().serialize(&mut ser).unwrap();
        println!("json str: {}", std::str::from_utf8(&bytes).unwrap());

        assert!(false)
    }
}

#[cfg(feature = "skipped")]
mod autoref2 {
    use super::*;

    macro_rules! encode {
        // (@bytes $serializer:expr, $cid:expr) => {{
        //     #[allow(unused_imports)]
        //     // use $crate::{DisplayKind, StdErrorKind};
        //     match $serializer {
        //         serializer => (&serializer)
        //             .encoder_kind()
        //             .serialize_bytes(serializer, $cid),
        //     }
        // }};
        (@link $serializer:expr, $cid:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            // match $serializer {
            //     serializer => (&&serializer)
            //         .encoder_kind()
            //         .serialize_link(serializer, $cid),
            // }
            (&$serializer)
                .encoder_kind()
                .serialize_link($serializer, $cid)
        }};
    }

    macro_rules! decode {
        (@any $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer).decoder_kind().deserialize_any(deserializer),
            }
        }};
        (@bytes $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer)
                    .decoder_kind()
                    .deserialize_bytes(deserializer),
            }
        }};
        (@byte_buf $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer)
                    .decoder_kind()
                    .deserialize_byte_buf(deserializer),
            }
        }};
        (@link $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer)
                    .decoder_kind()
                    .deserialize_link(deserializer),
            }
        }};
    }

    #[derive(Debug, Default)]
    struct TestCid(Cid);
    impl Serialize for TestCid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // encode!(@link serializer, &self.0)
            unimplemented!()
            // if <S as SerdeCodec<false, 0>>
        }
    }
    trait SerdeCodec<const C: u64>: Serializer {}
    // impl<const C: u64, S: Serializer> SerdeCodec<false, C> for S {}
    impl<W: std::io::Write> SerdeCodec<{ DagJson::CODE }> for &mut serde_json::Serializer<W> {}

    // trait SerdeCodec<const C: u64>: Serializer {}

    // Requires one extra autoref to call! Lower priority than XXXEncoderKind.
    trait GenericEncoderKind {
        fn encoder_kind(&self) -> GenericEncoder {
            println!("found generic kind");
            GenericEncoder
        }
    }
    impl<S> GenericEncoderKind for &S where S: Serializer {}
    struct GenericEncoder;
    impl GenericEncoder {
        fn serialize_link<S: SerdeCodec<false, 0>>(
            self,
            s: S,
            cid: &Cid,
        ) -> Result<S::Ok, S::Error> {
            println!("generic serialize_link");
            s.serialize_bytes(cid.to_bytes().as_slice())
        }
    }

    // Does not require any autoref if called as (&serializer).encoder_kind().
    trait DagJsonEncoderKind {
        fn encoder_kind(&self) -> DagJsonEncoder {
            println!("found dagjson kind");
            DagJsonEncoder
        }
    }

    impl<S> DagJsonEncoderKind for S where S: SerdeCodec<true, { DagJson::CODE }> {}
    struct DagJsonEncoder;
    impl DagJsonEncoder {
        fn serialize_link<S: SerdeCodec<true, { DagJson::CODE }>>(
            self,
            s: S,
            cid: &Cid,
        ) -> Result<S::Ok, S::Error> {
            println!("dagjson serialize_link");
            let cid_str = cid.to_string().map_err(S::Error::custom)?;
            s.serialize_newtype_variant("", 0, DagJson::SPECIAL_KEY, &cid_str)
        }
    }

    // should be serde_json...?

    // impl<'a, W: std::io::Write> Encoder<&'a mut serde_json::Serializer<W>> {
    //     fn from_dag_json(s: &'a mut serde_json::Serializer<W>) -> Self {
    //         Self(s)
    //     }
    // }

    #[test]
    fn test_cid_generic() {
        println!("cid bytes: {:?}", Cid::default().to_bytes().as_slice());

        let mut bytes = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap_err();
        TestCid::default().serialize(&mut ser).unwrap();
        println!("cbor bytes: {:?}", &bytes);

        assert!(false)
    }

    #[test]
    fn test_cid_dag_json() {
        let mut bytes = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap();
        TestCid::default().serialize(&mut ser).unwrap();
        println!("json str: {}", std::str::from_utf8(&bytes).unwrap());

        assert!(false)
    }
}

#[cfg(feature = "skipped")]
mod specialize {
    use std::any::TypeId;

    use super::*;

    macro_rules! encode {
        // (@bytes $serializer:expr, $cid:expr) => {{
        //     #[allow(unused_imports)]
        //     // use $crate::{DisplayKind, StdErrorKind};
        //     match $serializer {
        //         serializer => (&serializer)
        //             .encoder_kind()
        //             .serialize_bytes(serializer, $cid),
        //     }
        // }};
        (@link $serializer:expr, $cid:expr) => {{
            // #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $serializer {
                serializer => (&&serializer)
                    .encoder_kind()
                    .serialize_link(serializer, $cid),
            }
            // (&$serializer)
            //     .encoder_kind()
            //     .serialize_link($serializer, $cid)
        }};
    }

    macro_rules! decode {
        (@any $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer).decoder_kind().deserialize_any(deserializer),
            }
        }};
        (@bytes $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer)
                    .decoder_kind()
                    .deserialize_bytes(deserializer),
            }
        }};
        (@byte_buf $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer)
                    .decoder_kind()
                    .deserialize_byte_buf(deserializer),
            }
        }};
        (@link $deserializer:expr) => {{
            #[allow(unused_imports)]
            // use $crate::{DisplayKind, StdErrorKind};
            match $deserializer {
                deserializer => (&deserializer)
                    .decoder_kind()
                    .deserialize_link(deserializer),
            }
        }};
    }

    #[derive(Debug, Default)]
    struct TestCid(Cid);
    impl Serialize for TestCid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // encode!(@link serializer, &self.0)
            if <S as SerdeCodec<true, { DagJson::CODE }>>::CODE.is_some() {
                unimplemented!("found dagjson serializer")
            } else {
                unimplemented!("found generic serializer")
            }
        }
    }

    trait SerdeCodec<const F: bool, const C: u64>: Serializer {
        const CODE: Option<u64> = if F { Some(C) } else { None };
    }
    impl<const C: u64, S: Serializer> SerdeCodec<false, C> for S {}
    impl<W: std::io::Write> SerdeCodec<true, { DagJson::CODE }> for &mut serde_json::Serializer<W> {}

    // struct Encoder<S>(S);

    // // Requires one extra autoref to call! Lower priority than XXXEncoderKind.
    // trait GenericEncoderKind {
    //     fn encoder_kind(&self) -> GenericEncoder {
    //         println!("found generic kind");
    //         GenericEncoder
    //     }
    // }
    // impl<S> GenericEncoderKind for &S where S: SerdeCodec<false, 0> {}
    // struct GenericEncoder;
    // impl GenericEncoder {
    //     fn serialize_link<S: SerdeCodec<false, 0>>(
    //         self,
    //         s: S,
    //         cid: &Cid,
    //     ) -> Result<S::Ok, S::Error> {
    //         println!("generic serialize_link");
    //         s.serialize_bytes(cid.to_bytes().as_slice())
    //     }
    // }

    // // Does not require any autoref if called as (&serializer).encoder_kind().
    // trait DagJsonEncoderKind {
    //     fn encoder_kind(&self) -> DagJsonEncoder {
    //         println!("found dagjson kind");
    //         DagJsonEncoder
    //     }
    // }

    // // impl<S> DagJsonEncoderKind for S where S: SerdeCodec<true, { DagJson::CODE }> {}
    // impl<W: std::io::Write> DagJsonEncoderKind for &mut serde_json::Serializer<W> {}
    // struct DagJsonEncoder;
    // impl DagJsonEncoder {
    //     fn serialize_link<S: SerdeCodec<true, { DagJson::CODE }>>(
    //         self,
    //         s: S,
    //         cid: &Cid,
    //     ) -> Result<S::Ok, S::Error> {
    //         println!("dagjson serialize_link");
    //         let cid_str = cid.to_string().map_err(S::Error::custom)?;
    //         s.serialize_newtype_variant("", 0, DagJson::SPECIAL_KEY, &cid_str)
    //     }
    // }

    #[test]
    fn test_cid_generic() {
        println!("cid bytes: {:?}", Cid::default().to_bytes().as_slice());

        let mut bytes = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap_err();
        TestCid::default().serialize(&mut ser).unwrap();
        println!("cbor bytes: {:?}", &bytes);

        assert!(false)
    }

    #[test]
    fn test_cid_dag_json() {
        let mut bytes = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut bytes);
        // (&mut &mut &mut Encoder(&mut ser))
        //     .serialize_link(&TestCid::default().0)
        //     .unwrap();
        TestCid::default().serialize(&mut ser).unwrap();
        println!("json str: {}", std::str::from_utf8(&bytes).unwrap());

        assert!(false)
    }
}

#[cfg(feature = "skipped")]
mod autoref3 {
    use super::*;

    #[derive(Debug, Default)]
    struct TestCid(Cid);
    impl Serialize for TestCid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            (&&Encoder::<S>::new()).serialize_link(serializer, &self.0)
        }
    }

    struct Encoder<S: Serializer>(std::marker::PhantomData<S>);
    impl<S: Serializer> Encoder<S> {
        pub fn new() -> Self {
            Self(std::marker::PhantomData)
        }
    }

    trait ViaGeneric {
        fn serialize_link<S: Serializer>(self, serializer: S, cid: &Cid)
            -> Result<S::Ok, S::Error>;
    }

    impl<T> ViaGeneric for &Encoder<T>
    where
        T: Serializer,
    {
        fn serialize_link<S: Serializer>(
            self,
            serializer: S,
            cid: &Cid,
        ) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(cid.to_bytes().as_slice())
            // Err(<Self::Error as ser::Error>::custom("use autoref"))
        }
    }

    #[test]
    fn test_cid_generic() {
        println!("cid bytes: {:?}", Cid::default().to_bytes().as_slice());

        let mut bytes = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut bytes);

        TestCid::default().serialize(&mut ser).unwrap();
        println!("cbor bytes: {:?}", &bytes);

        assert!(false)
    }

    trait Tag<const C: u64>: Serializer {}
    impl<'a, W: std::io::Write> Tag<{ DagJson::CODE }> for &'a mut serde_json::Serializer<W> {}

    trait ViaDagJson {
        fn serialize_link<S: Serializer>(self, serializer: S, cid: &Cid)
            -> Result<S::Ok, S::Error>;
    }
    impl<T> ViaDagJson for &&Encoder<T>
    where
        T: Tag<{ DagJson::CODE }>,
    {
        fn serialize_link<S: Serializer>(
            self,
            serializer: S,
            cid: &Cid,
        ) -> Result<S::Ok, S::Error> {
            let cid_str = cid.to_string().map_err(S::Error::custom)?;
            serializer.serialize_newtype_variant("", 0, DagJson::IPLD_KEY, &cid_str)
        }
    }

    #[test]
    fn test_cid_dag_json() {
        let mut bytes = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut bytes);

        TestCid::default().serialize(&mut ser).unwrap();
        println!("json str: {}", std::str::from_utf8(&bytes).unwrap());

        assert!(false)
    }
}
