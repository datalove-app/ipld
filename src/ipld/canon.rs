//! Canonicalization conversion implementations.
//!
//!

use crate::{borrowed::Ipld, CodecExt, Error};
use cid::Cid;
use failure::format_err;
use std::{convert::TryFrom, marker::PhantomData};

/// Shorthand for deriving `From<& _>` for a reference to a type.
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! borrowed_ipld_from_ref {
    ($type:ty : $member:ident) => {
        impl<'a> From<&'a $type> for Ipld<'a>
        where
            C: CodecExt,
        {
            #[inline]
            fn from(t: &'a $type) -> Ipld<'a> {
                Ipld::$member(*t)
            }
        }
    };
}

// null

impl<'a> TryFrom<Ipld<'a>> for ()
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Null(_) => Ok(()),
            _ => Err(Error::Ipld(format_err!("Not Null"))),
        }
    }
}

impl<'a> From<&'a ()> for Ipld<'a>
where
    C: CodecExt,
{
    #[inline]
    fn from(_: &'a ()) -> Self {
        Ipld::Null(PhantomData)
    }
}

// bool

borrowed_ipld_from_ref!(bool: Bool);
impl<'a> TryFrom<Ipld<'a>> for bool
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Bool(b) => Ok(b),
            _ => Err(Error::Ipld(format_err!("Not Bool"))),
        }
    }
}

// int

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! try_from_num {
    ($type:ty : $member:ident) => {
        impl<'a> TryFrom<Ipld<'a>> for $type
        where
            C: CodecExt,
        {
            type Error = Error;

            #[inline]
            fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
                match ipld {
                    Ipld::$member(i) => Ok(i),
                    _ => Err(Error::Ipld(::failure::format_err!(
                        "Not {}",
                        ::std::stringify!($type)
                    ))),
                }
            }
        }

        borrowed_ipld_from_ref!($type: $member);
    };
}

try_from_num!(i8: Int8);
try_from_num!(i16: Int16);
try_from_num!(i32: Int32);
try_from_num!(i64: Int64);
try_from_num!(i128: Int128);
try_from_num!(u8: Uint8);
try_from_num!(u16: Uint16);
try_from_num!(u32: Uint32);
try_from_num!(u64: Uint64);
try_from_num!(u128: Uint128);
try_from_num!(f32: Float32);
try_from_num!(f64: Float64);

// string

impl<'a> TryFrom<Ipld<'a>> for &'a str
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(s) => Ok(s),
            _ => Err(Error::Ipld(format_err!("Not String"))),
        }
    }
}

impl<'a> TryFrom<Ipld<'a>> for String
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(s) => Ok(s.into()),
            _ => Err(Error::Ipld(format_err!("Not String"))),
        }
    }
}

impl<'a> From<&'a String> for Ipld<'a>
where
    C: CodecExt,
{
    #[inline]
    fn from(s: &'a String) -> Self {
        Ipld::String(&*s)
    }
}

// bytes

impl<'a> TryFrom<Ipld<'a>> for &'a [u8]
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Bytes(b) => Ok(b),
            _ => Err(Error::Ipld(format_err!("Not Bytes"))),
        }
    }
}

impl<'a> From<&'a Box<[u8]>> for Ipld<'a>
where
    C: CodecExt,
{
    #[inline]
    fn from(bytes: &'a Box<[u8]>) -> Self {
        Ipld::Bytes(&*bytes)
    }
}

#[cfg(feature = "bytes_")]
impl<'a> TryFrom<Ipld<'a>> for bytes::Bytes
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Bytes(b) => Ok(bytes::Bytes::copy_from_slice(b)),
            _ => Err(Error::Ipld(format_err!("Not Bytes"))),
        }
    }
}

#[cfg(feature = "bytes_")]
impl<'a> From<&'a bytes::Bytes> for Ipld<'a>
where
    C: CodecExt,
{
    #[inline]
    fn from(bytes: &'a bytes::Bytes) -> Self {
        Ipld::Bytes(bytes.as_ref())
    }
}

// cid

impl<'a> TryFrom<Ipld<'a>> for Cid
where
    C: CodecExt,
{
    type Error = Error;

    #[inline]
    fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Link(cid) => Ok(cid),
            _ => Err(Error::Ipld(format_err!("Not Link"))),
        }
    }
}

impl<'a> From<&'a Cid> for Ipld<'a>
where
    C: CodecExt,
{
    #[inline]
    fn from(link: &'a Cid) -> Self {
        Ipld::Link(link.to_owned())
    }
}

/// Derives `TryFrom<Ipld>` for an arbitrary struct.
/// Used within other macros that produce an IPLD `Representation` for custom types.
#[macro_export]
macro_rules! derive_ipld_for_struct {
    ($name:ident { $($member:ident : $value_type:ty,)* }) => {
        ::paste::item! {
            #[doc(hidden)]
            enum [<$name Field>] {
                $([<Field $member>],)*
            }

            impl [<$name Field>] {
                #[inline]
                fn field(key: &str) -> Result<Self, Error> {
                    match key {
                        $(::std::stringify!($member) => Ok(Self::$member),)*
                        _ => Err(Error::Codec(::failure::format_err!("Missing key: {}", key).into())),
                    }
                }
            }
        }
        ::paste::item! {
            #[doc(hidden)]
            #[derive(Default)]
            struct [<$name Builder>] {
                $($member : Option<$value_type>,)*
            }

            impl [<$name Builder>] {
                $(
                    #[inline]
                    fn [<set_ $member>](&mut self, value: $value_type) -> &mut Self {
                        self.$member = Some(value);
                        self
                    }
                )*

                #[inline]
                fn build(self) -> Result<$name, Error> {
                    Ok($name {
                        $($member: self.$member
                            .ok_or(Error::Codec(::failure::format_err!("Missing key: {}", ::std::stringify!($member))))?,)*
                    })
                }
            }
        }

        ::paste::item! {
            impl<'a> TryFrom<Ipld<'a>> for $name
            where
                C: CodecExt,
            {
                type Error = Error;

                #[inline]
                fn try_from(ipld: Ipld<'a>) -> Result<Self, Self::Error> {
                    match ipld {
                        Ipld::Map(map_iter) => match map_iter {
                            IpldMapIter::Vec(iter) => {
                                let mut iter = iter.into_inner();
                                let mut builder = [<$name Builder>]::default();

                                while let Some((key, value)) = iter.next() {
                                    match [<$name Builder>]::field(key)? {
                                        $([<$name Field>]::[<Field $member>] => {
                                            builder.[<set_ $member>]($value_type::try_from(value)?)
                                        },)*
                                    };
                                };

                                builder.build()
                            }
                        },
                        _ => Err(Error::Ipld(IpldError::NotMap)),
                    }
                }
            }
        }

        ::paste::item! {
            impl<'a> TryInto<Ipld<'a>> for &'a $name
            where
                C: CodecExt,
            {
                type Error = Error;

                #[inline]
                fn try_into(self) -> Result<Ipld<'a>, Self::Error> {


    //                let mut map: BTreeMap<&'static str, Ipld<'a>> = BTreeMap::new();
    //                $(map.insert(::std::stringify!($member), (&(self.$member)).try_into()?);)*
    //                Ok(Ipld::Map(map))
                }
            }
        }
    };
    // (@count $t1:tt, $($t:tt),+) => { 1 + derive_ipld_for_struct!(@count $($t),+) };
    // (@count $t:tt) => { 1 };
}

#[macro_export]
macro_rules! derive_ipld_for_enum {
    ($name:ident { $(| $member:ident,)* }) => {};
}

#[macro_export]
macro_rules! derive_ipld_for_union {
    ($name:ident { $($member:ident : $value_type:ty,)* }) => {};
}
