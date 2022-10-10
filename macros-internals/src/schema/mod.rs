//! `schema!` macro.

mod display;
pub(crate) mod expand;
pub(crate) mod parse;

mod compound;
mod r#enum;
mod primitive;
mod r#struct;
mod union;

pub use compound::*;
pub use expand::*;
pub use kind::{type_kinds, SchemaKind, TypedKind};
pub use parse::*;
pub use primitive::*;
pub use r#enum::*;
pub use r#struct::*;
pub use union::*;

use crate::{common::*, dev::*};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use std::ops::{BitOr, Deref};
use syn::{parse_str, Attribute, Generics, Ident, LitStr, Path, Type, Visibility};

///
#[derive(Debug)]
pub struct SchemaDefinition {
    meta: SchemaMeta,
    repr: ReprDefinition,
}

///
#[derive(Debug, Clone)]
pub struct SchemaMeta {
    pub lib: TokenStream,
    pub typedef_str: String,
    pub internal: bool,
    pub try_from: Option<LitStr>,
    pub attrs: OuterAttributes,
    pub vis: Visibility,
    pub name: Ident,
    pub generics: Option<Generics>,
}

impl SchemaMeta {
    /// Creates a `TokenStream` of the `ipld` lib used (either `crate` or the `ipld` crate name).
    pub fn lib(is_internal: bool) -> TokenStream {
        if is_internal {
            return quote!(crate);
        }

        let path = crate_name(attr::IPLD_CRATE_NAME).map_or(
            Path::from(Ident::new(attr::IPLD_CRATE_NAME, Span::call_site())),
            |found_crate_name| {
                match found_crate_name {
                    FoundCrate::Itself => parse_str::<Path>(attr::IPLD_CRATE_NAME),
                    FoundCrate::Name(name) => parse_str::<Path>(&name),
                }
                .expect("`ipld` is not present in Cargo.toml")
            },
        );

        quote!(#path)
    }

    pub fn generics_tokens(&self) -> TokenStream {
        self.generics
            .as_ref()
            .map(|g| quote!(#g))
            .unwrap_or(TokenStream::default())
    }

    // fn to_try_from_meta(&self) -> Self {
    //     SchemaMeta {
    //         typedef_str: String::new(),
    //         ipld_schema_lib: self.ipld_schema_lib.clone(),
    //         try_from: None,
    //         attrs: self.attrs.clone(),
    //         vis: Visibility::Inherited,
    //         name: self.try_from_name(),
    //         generics: self.generics.clone(),
    //     }
    // }
    //
    // fn try_from_name(&self) -> Ident {
    //     let try_from_name = self.try_from.as_ref().unwrap().value();
    //     Ident::new(&try_from_name, Span::call_site())
    // }
}

// #[derive(Debug, Clone)]
// pub struct SchemaAttrs {
//     lib: Path,
//     try_from: Option<LitStr>,
// }

///
#[derive(Debug)]
pub enum ReprDefinition {
    Null(NullReprDefinition),
    Bool(BoolReprDefinition),
    Int(IntReprDefinition),
    Float(FloatReprDefinition),
    String(StringReprDefinition),
    Bytes(BytesReprDefinition),
    List(ListReprDefinition),
    Map(MapReprDefinition),
    Link(LinkReprDefinition),
    Struct(StructReprDefinition),
    Enum(EnumReprDefinition),
    Union(UnionReprDefinition),
    Copy(CopyReprDefinition),
}

impl ReprDefinition {
    fn supports_try_from(&self) -> bool {
        match self {
            Self::Int(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::Bytes(BytesReprDefinition::Basic) => true,
            _ => false,
        }
    }
}

// #[macro_use]
pub mod kind {
    // use super::*;
    use std::ops::{BitAnd, BitOr};
    use type_kinds::*;
    use typenum::*;
    use typenum_macro::*;

    ///
    /// TODO: BitAnd/BitOr dont work without fully-qualified syntax
    pub trait TypedKind
    where
        Self: NonZero
            + Unsigned
            // + IsLessOrEqual<All> // TODO: replace with And<Self, All>: IsEqual<All>,
            // + BitAnd<Self>
            // + BitAnd<All>
            + BitAnd<Scalar>
            + BitAnd<Recursive>
            + BitAnd<Any>
            + BitAnd<Schema>
            // + BitAnd<TypedInt>
            // + BitAnd<TypedFloat>
            + BitOr<Null>,
    {
        const KIND: SchemaKind;
        // const IS_SCALAR: bool = SchemaKind::Scalar.contains(Self::KIND);
        // const IS_RECURSIVE: bool = SchemaKind::Recursive.contains(Self::KIND);
        const IS_DATA_MODEL: bool = SchemaKind::Any.contains(Self::KIND);
        // const IS_SCHEMA: bool = SchemaKind::Schema.contains(Self::KIND) || Self::IS_TYPED_NUM;
        // const IS_TYPED_NUM: bool = SchemaKind::TypedNum.contains(Self::KIND) && !Self::IS_VARIOUS;
        const IS_VARIOUS: bool = !is_unary::<Self>();
    }

    impl<T> TypedKind for T
    where
        T: NonZero
            + Unsigned
            // + IsLessOrEqual<All>
            // + BitAnd<Self>
            // + BitAnd<All>
            + BitAnd<Scalar>
            + BitAnd<Recursive>
            + BitAnd<Any>
            + BitAnd<Schema>
            // + BitAnd<TypedInt>
            // + BitAnd<TypedFloat>
            + BitOr<Null>,
        // And<Self, All>: IsEqual<All>,
    {
        const KIND: SchemaKind = SchemaKind::from_bits_truncate(T::U32);
    }

    // pub trait DataModelKind: TypedKind
    // where
    //     And<Self, Any>: IsEqual<Any>,
    // {
    // }
    // impl<T> DataModelKind for T
    // where
    //     T: TypedKind,
    //     // T: IsLessOrEqual<Any>,
    //     // T: BitAnd<Any>,
    //     And<T, Any>: IsEqual<Any>,
    // {
    // }

    macro_rules! def_kind { (
        $($name:ident = $b:expr;)*) => {
            bitflags::bitflags! {
                /// Enum of possible [Data Model](), [Schema]() and [Representation]() kinds.
                ///
                #[repr(transparent)]
                pub struct SchemaKind: u32 {
                    $(
                        const $name = $b;
                    )*

                    /// Marker flag for scalar data model kinds.
                    const Scalar = Self::Null.bits
                        | Self::Bool.bits
                        | Self::Int.bits
                        | Self::Float.bits
                        | Self::String.bits
                        | Self::Bytes.bits
                        | Self::Link.bits;

                        /// Marker flag for scalar data model kinds.
                    const Recursive = Self::List.bits | Self::Map.bits;

                    /// Marker flag for any and all valid data model kinds.
                    const Any = Self::Scalar.bits | Self::Recursive.bits;

                    /// Marker flag for any data model or schema kind.
                    const Schema = Self::Any.bits
                        | Self::Struct.bits
                        | Self::Enum.bits
                        | Self::Union.bits
                        | Self::Copy.bits
                        | Self::Advanced.bits;

                    /// Marker flag for lib-specific schema types for integers.
                    const TypedInt = Self::Int8.bits
                        | Self::Int16.bits
                        | Self::Int32.bits
                        | Self::Int64.bits
                        | Self::Int128.bits
                        | Self::Uint8.bits
                        | Self::Uint16.bits
                        | Self::Uint32.bits
                        | Self::Uint64.bits
                        | Self::Uint128.bits;

                    /// Marker flag for lib-specific schema types for
                    /// floating-point numbers.
                    const TypedFloat = Self::Float32.bits | Self::Float64.bits;

                    ///
                    const TypedScalar = Self::Scalar.bits
                        | Self::TypedInt.bits
                        | Self::TypedFloat.bits;
                }
            }

            /// [`typenum`] types representing known [`SchemaKind`]s.
            // #[macro_use]
            pub mod type_kinds {
                use super::*;
                // use typenum;
                // use $crate::dev::typenum_macro;

                $(pub type $name = tyuint!($b);)*

                // pub type Optional<T> = typenum::op!(Null | T);

                pub type Scalar = op!(Null | Bool | Int | Float | String | Bytes | Link);
                pub type Recursive = op!(List | Map);
                pub type Any = op!(Scalar | Recursive);

                pub type Schema = op!(Any | Struct | Enum | Union | Copy | Advanced);
                pub type TypedInt = op!(Int8 | Int16 | Int32 | Int64 | Int128 | Uint8 | Uint16 | Uint32 | Uint64 | Uint128);
                pub type TypedFloat = op!(Float32 | Float64);

                pub type TypedScalar = op!(Scalar | TypedInt | TypedFloat);

                #[doc(hidden)]
                pub type All = op!(Any | Schema | TypedInt | TypedFloat);
                #[doc(hidden)]
                pub type Empty = U0;
            }
        };
    }

    def_kind! {
        Null =      0b0001;
        Bool =      0b0010;
        Int =       0b0100;
        Float =     0b1000;
        String =    0b0001_0000;
        Bytes =     0b0010_0000;
        List =      0b0100_0000;
        Map =       0b1000_0000;
        Link =      0b0001_0000_0000;
        //
        Struct =    0b0010_0000_0000;
        Enum =      0b0100_0000_0000;
        Union =     0b1000_0000_0000;
        Copy =      0b0001_0000_0000_0000;
        Advanced =  0b0010_0000_0000_0000;
        //
        Int8 =      0b0100_0000_0000_0000;
        Int16 =     0b1000_0000_0000_0000;
        Int32 =     0b0001_0000_0000_0000_0000;
        Int64 =     0b0010_0000_0000_0000_0000;
        Int128 =    0b0100_0000_0000_0000_0000;
        Uint8 =     0b1000_0000_0000_0000_0000;
        Uint16 =    0b0001_0000_0000_0000_0000_0000;
        Uint32 =    0b0010_0000_0000_0000_0000_0000;
        Uint64 =    0b0100_0000_0000_0000_0000_0000;
        Uint128 =   0b1000_0000_0000_0000_0000_0000;
        Float32 =   0b0001_0000_0000_0000_0000_0000_0000;
        Float64 =   0b0010_0000_0000_0000_0000_0000_0000;
    }

    impl SchemaKind {
        /// Const function for determining equality between [`Kind`]s.
        #[doc(hidden)]
        pub const fn eq(&self, other: &Self) -> bool {
            match (*self, *other) {
                (Self::Null, Self::Null)
                | (Self::Bool, Self::Bool)
                | (Self::Int, Self::Int)
                | (Self::Int8, Self::Int8)
                | (Self::Int16, Self::Int16)
                | (Self::Int32, Self::Int32)
                | (Self::Int64, Self::Int64)
                | (Self::Int128, Self::Int128)
                | (Self::Uint8, Self::Uint8)
                | (Self::Uint16, Self::Uint16)
                | (Self::Uint32, Self::Uint32)
                | (Self::Uint64, Self::Uint64)
                | (Self::Uint128, Self::Uint128)
                | (Self::Float, Self::Float)
                | (Self::Float32, Self::Float32)
                | (Self::Float64, Self::Float64)
                | (Self::String, Self::String)
                | (Self::Bytes, Self::Bytes)
                | (Self::List, Self::List)
                | (Self::Map, Self::Map)
                | (Self::Link, Self::Link)
                | (Self::Struct, Self::Struct)
                | (Self::Enum, Self::Enum)
                | (Self::Union, Self::Union)
                | (Self::Copy, Self::Copy) => true,
                _ => false,
            }
        }

        ///
        pub const fn is_link(&self) -> bool {
            self.contains(Self::Link)
        }

        ///
        pub const fn is_dm(&self) -> bool {
            Self::Any.contains(*self)
        }

        pub const fn from<const K: u32>() -> Self {
            Self::from_bits_truncate(K)
        }

        // ///
        // pub const fn is_
    }

    const fn is_unary<T: TypedKind>() -> bool {
        T::KIND.bits.count_ones() == 1
    }

    // pub trait BitOps<U = Self>: BitAnd<U> + BitOr<U> {}
    // impl<T, U> BitOps<U> for T where T: BitAnd<U> + BitOr<U> {}
}

// #[derive(Debug)]
// pub struct ContainerAttr {
//     internal: bool,
//     try_from: Option<LitStr>,
// }

/// Wrapper around a vec of fields.
#[derive(Debug)]
pub struct Fields<T>(Vec<T>);
impl<T> Deref for Fields<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type SchemaAttrs = std::collections::HashSet<SchemaAttr>;

#[derive(Debug, PartialEq)]
pub enum SchemaAttr {
    Internal,
    TryFrom(LitStr),
    Wrapper(Type),
}

// #[derive(Debug)]
// pub struct FieldAttr {
//     wrapper: Option<Ident>,
// }

/// Keywords unique to IPLD Schemas and Representations
#[macro_use]
pub(crate) mod kw {
    // Helper macro for parsing a keyword-argument pair from IPLD schemas.
    #[macro_export(local_inner_macros)]
    macro_rules! parse_kwarg {
        ($input:ident, $kw:ident => $type:ty) => {{
            $input.parse::<crate::schema::kw::$kw>()?;
            $input.parse::<$type>()?
        }};
    }

    crate::define_keywords! {
        // main keywords
        representation advanced
        // schema data types
        null bool boolean bytes string list map link copy
        int uint8 uint16 uint32 uint64 uint128 int8 int16 int32 int64 int128
        float float32 float64
        // representation types
        listpairs stringjoin stringpairs
        tuple keyed kinded envelope inline byteprefix
        // representation type args
        field nullable optional implicit rename
        join innerDelim entryDelim
        fieldOrder discriminantKey contentKey

        // custom container and field attributes
        internal wrapper ctx_bounds
    }

    // pub struct Directive<K, T>(pub Option<T>, pub std::marker::PhantomData<K>);
}

// Exported Attributes
#[doc(hidden)]
#[macro_export]
macro_rules! def_attributes {
    ($(#[$meta:meta])* $attr:ident) => {
        $(#[$meta])*
        #[proc_macro_attribute]
        pub fn $attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    };
    ($attr:ident) => {
        #[proc_macro_attribute]
        pub fn $attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    };
}

// struct Methods(Vec<ItemFn>);
