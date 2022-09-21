use super::*;
use crate::dev::{
    schema::{
        expand::{self, ExpandBasicRepresentation},
        kw, SchemaKind,
    },
    SchemaMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for KindedUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let name = &meta.name;
        let fields = self.iter().map(UnionField::<SchemaKind>::typedef);

        // TODO: assert that all nested REPR_KINDs are unique and are DM kinds

        quote! {
            #(#attrs)*
            #[derive(Deserialize, Serialize)]
            #[serde(untagged)]
            #vis enum #name {
                #(#fields,)*
            }
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let dm_kind = self.dm_kind();
        let repr_kind = self.repr_kind();
        let name_branches = self.iter().map(UnionField::<SchemaKind>::name_branch);
        // let kind_branches: Vec<TokenStream> = self.iter().map(|f| f.kind_branch(&lib)).collect();

        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = #dm_kind;
                const SCHEMA_KIND: Kind = Kind::Union;
                const REPR_KIND: Kind = #repr_kind;
                // const FIELDS: Fields = Fields::Keyed(&[#(#fields,)*]);

                #[inline]
                fn name(&self) -> &'static str {
                    match self {
                        #(#name_branches,)*
                    }
                }

                // #[inline]
                // fn kind(&self) -> Kind {
                //     match self {
                //         #(#kind_branches,)*
                //     }
                // }
            },
        )
    }

    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let name = &meta.name;
        let non_link_visitors = self
            .iter()
            .filter(|f| f.key != SchemaKind::Link)
            .map(|f| f.visit_fn(meta))
            .flatten();
        let link_visitors = self
            .iter()
            .filter(|f| f.key == SchemaKind::Link)
            .map(|f| f.visit_fn(meta))
            .flatten();

        quote! {
            // TODO: add a method that does the delegation to the variant, rather than codegen each
            //
            // impl<'a, C: Context> ContextSeed<'a, C, #name> {
            //     fn visit_primitive<V, E>(self, v: V) -> Result<(), E>
            //     where
            //         V: serde::de::IntoDeserializer,
            //         E: serde::de::Error,
            //     {
            //         let seed = #lib::dev::macros::impl_selector_seed_serde! {
            //             @selector_seed_wrap
            //             self #constructor => #ty
            //         };
            //         seed.deserialize(v.into_deserializer())
            //     }
            //
            //     fn visit_list<A>(self, v: V) -> Result<(), A::Error>
            //     where
            //         A: serde::de::SeqAccess,
            //     {
            //         let seed = #lib::dev::macros::impl_selector_seed_serde! {
            //             @selector_seed_wrap
            //             self #constructor => #ty
            //         };
            //         seed.deserialize(v.into_deserializer())
            //     }
            // }

            #lib::dev::macros::impl_selector_seed_serde! {
                @codec_seed_visitor {} {} #name
            {
                #[inline]
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "A `{}`", <#name as Representation>::NAME)
                }

                #(#non_link_visitors)*
            }}
            #lib::dev::macros::impl_selector_seed_serde! {
                @codec_seed_visitor_ext {} {} #name
            {
                #(#link_visitors)*
            }}
            #lib::dev::macros::impl_selector_seed_serde! {
                @selector_seed_codec_deseed @any {} {} #name
            }
            #lib::dev::macros::impl_selector_seed_serde! {
                @selector_seed_select {} {} #name
            }
        }
    }

    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}

impl KindedUnionReprDefinition {
    fn dm_kind(&self) -> TokenStream {
        self.iter()
            .map(|f| f.dm_kind())
            .fold(quote!(Kind::empty()), |ts, kind| quote!(#ts.union(#kind)))
    }

    fn repr_kind(&self) -> TokenStream {
        self.iter()
            .map(|f| f.ty(false))
            .map(|ty| quote!(<#ty as Representation>::REPR_KIND))
            .fold(quote!(Kind::empty()), |ts, kind| quote!(#ts.union(#kind)))
    }
}

impl UnionField<SchemaKind> {
    const NULL: &'static str = "Null";
    const BOOL: &'static str = "Bool";
    const INT: &'static str = "Int";
    // const INT8: &'static str = "Int8";
    // const INT16: &'static str = "Int16";
    // const INT32: &'static str = "In32t";
    // const INT64: &'static str = "Int64";
    // const INT128: &'static str = "Int128";
    // const UINT8: &'static str = "Uint8";
    // const UINT16: &'static str = "Uint16";
    // const UINT32: &'static str = "Uint32";
    // const UINT64: &'static str = "Uint64";
    // const UINT128: &'static str = "Uint128";
    const FLOAT: &'static str = "Float";
    // const FLOAT32: &'static str = "Float32";
    // const FLOAT64: &'static str = "Float64";
    const BYTES: &'static str = "Bytes";
    const STRING: &'static str = "String";
    const LIST: &'static str = "List";
    const MAP: &'static str = "Map";
    const LINK: &'static str = "Link";

    /// Outputs the kinded enum variant name.
    /// TODO: this is wrong, let users define the name and just check the repr_kind
    fn name(&self) -> Ident {
        let name = match self.key {
            SchemaKind::Null => Self::NULL,
            SchemaKind::Bool => Self::BOOL,
            SchemaKind::Int => Self::INT,
            SchemaKind::Float => Self::FLOAT,
            SchemaKind::Bytes => Self::BYTES,
            SchemaKind::String => Self::STRING,
            SchemaKind::List => Self::LIST,
            SchemaKind::Map => Self::MAP,
            SchemaKind::Link => Self::LINK,
            _ => unreachable!(),
        };

        Ident::new(name, Span::call_site())
    }

    fn dm_kind(&self) -> TokenStream {
        match self.key {
            SchemaKind::Null => quote!(Kind::Null),
            SchemaKind::Bool => quote!(Kind::Bool),
            SchemaKind::Int => quote!(Kind::Int),
            SchemaKind::Float => quote!(Kind::Float),
            SchemaKind::String => quote!(Kind::String),
            SchemaKind::Bytes => quote!(Kind::Bytes),
            SchemaKind::List => quote!(Kind::List),
            SchemaKind::Map => quote!(Kind::Map),
            SchemaKind::Link => quote!(Kind::Link),
            _ => unreachable!(),
        }
    }

    fn ty(&self, with_wrapper: bool) -> TokenStream {
        let generics = &self.generics;
        let ty = match self.key {
            SchemaKind::String => Ident::new("IpldString", Span::call_site()),
            _ => self.value.clone(),
        };

        self.linked
            .then(|| quote!(Link<#ty #generics>))
            .or_else(|| Some(quote!(#ty #generics)))
            .map(|ty| match &self.wrapper {
                Some(wrapper) if with_wrapper => quote!(#wrapper <#ty>),
                _ => ty,
            })
            .unwrap()
    }

    fn typedef(&self) -> TokenStream {
        let attrs = &self.attrs;
        let name = &self.name();
        let ty = self.ty(true);

        // let implicit_attr = if let Some(implicit) = &self.implicit {
        //     quote!(#[serde(default)])
        // } else {
        //     TokenStream::default()
        // };
        //
        // let rename_attr = if let Some(rename) = &self.rename {
        //     quote!(#[serde(rename = #rename)])
        // } else {
        //     TokenStream::default()
        // };

        quote! {
            #(#attrs)*
            #name(#ty)
        }
    }

    fn name_branch(&self) -> TokenStream {
        let name = self.name();
        // let ty = &self.value;
        quote!(Self::#name(ty) => Representation::name(ty))
    }

    fn dm_kind_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::data_model_kind(ty))
    }

    fn schema_kind_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::schema_kind(ty))
    }

    fn repr_kind_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::repr_kind(ty))
    }
}

impl UnionField<SchemaKind> {
    fn visit_params(&self) -> Vec<(TokenStream, TokenStream)> {
        match &self.key {
            SchemaKind::Null => vec![
                (quote!(visit_unit), quote!()),
                (quote!(visit_none), quote!()),
            ],
            SchemaKind::Bool => vec![(quote!(visit_bool), quote!(v: bool))],
            SchemaKind::Int => vec![
                (quote!(visit_u8), quote!(v: u8)),
                (quote!(visit_u16), quote!(v: u16)),
                (quote!(visit_u32), quote!(v: u32)),
                (quote!(visit_u64), quote!(v: u64)),
                (quote!(visit_u128), quote!(v: u128)),
                (quote!(visit_i8), quote!(v: i8)),
                (quote!(visit_i16), quote!(v: i16)),
                (quote!(visit_i32), quote!(v: i32)),
                (quote!(visit_i64), quote!(v: i64)),
                (quote!(visit_i128), quote!(v: i128)),
            ],
            SchemaKind::Float => vec![
                (quote!(visit_f32), quote!(v: f32)),
                (quote!(visit_f64), quote!(v: f64)),
            ],
            SchemaKind::Bytes => vec![
                (quote!(visit_bytes), quote!(v: &[u8])),
                (quote!(visit_byte_buf), quote!(v: Vec<u8>)),
            ],
            SchemaKind::String => vec![
                (quote!(visit_str), quote!(v: &str)),
                (quote!(visit_string), quote!(v: String)),
            ],
            SchemaKind::List => vec![(quote!(visit_seq), quote!(v: A))],
            SchemaKind::Map => vec![(quote!(visit_map), quote!(v: A))],
            SchemaKind::Link => vec![
                (quote!(visit_link_bytes), quote!(v: &[u8])),
                (quote!(visit_link_str), quote!(v: &str)),
            ],
            _ => unreachable!(),
        }
    }

    fn visit_fn(&self, meta: &SchemaMeta) -> impl Iterator<Item = TokenStream> + '_ {
        let lib = &meta.lib;
        let name = &meta.name;
        let field_name = self.name();
        let ty = self.ty(false);
        let visit_impl = {
            let deserializer = match &self.key {
                SchemaKind::Null => quote!(().into_deserializer()),
                SchemaKind::Bytes => quote!(serde::de::value::BytesDeserializer::new(&v)),
                SchemaKind::List => quote!(serde::de::value::SeqAccessDeserializer::<A>::new(v)),
                SchemaKind::Map => quote!(serde::de::value::MapAccessDeserializer::<A>::new(v)),
                _ => quote!(v.into_deserializer()),
            };

            quote! {
                // let seed = #lib::dev::macros::impl_selector_seed_serde! {
                //     @selector_seed_wrap
                //     self { #name::#field_name => #ty }
                // };
                // seed.deserialize::<C, _>(#deserializer)
                unimplemented!()
            }
        };

        self.visit_params()
            .into_iter()
            .map(move |(visit_fn, args)| match self.key {
                SchemaKind::List => quote! {
                    #[inline]
                    fn #visit_fn<A>(self, #args) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>
                    {
                        #visit_impl
                    }
                },
                SchemaKind::Map => quote! {
                    #[inline]
                    fn #visit_fn<A>(self, #args) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>
                    {
                        #visit_impl
                    }
                },
                _ => quote! {
                    #[inline]
                    fn #visit_fn<E>(self, #args) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        #visit_impl
                    }
                },
            })
    }
}

impl SchemaKind {}
