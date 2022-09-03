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
        let lib = &meta.lib;
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let ident = &meta.name;
        let fields: Vec<TokenStream> = self
            .iter()
            .map(UnionField::<SchemaKind>::field_typedef)
            .collect();

        quote! {
            #(#attrs)*
            #[derive(serde::Deserialize, serde::Serialize)]
            #[serde(untagged)]
            #vis enum #ident {
                #(#fields,)*
            }
        }
    }

    // fn derive_serde(&self, meta: &SchemaMeta) -> TokenStream {
    //     TokenStream::default()
    // }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let name_branches: Vec<TokenStream> = self.iter().map(|f| f.name_branch(&lib)).collect();
        let kind_branches: Vec<TokenStream> = self.iter().map(|f| f.kind_branch(&lib)).collect();

        expand::impl_repr(
            meta,
            quote! {
                const DATA_MODEL_KIND: Kind = unimplemented!();
                const SCHEMA_KIND: Kind = Kind::Union;
                const REPR_KIND: Kind = unimplemented!();
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
        expand::impl_select(
            meta,
            // quote! {
            //     // unimplemented!()
            //     Ok(Null::r#match(seed)?.map(|_| Self))
            // },
            quote! {
                unimplemented!()

                // if #lib::dev::type_eq::<Self, S>() {
                //     type_cast_selection::<Self, S, _, _>(|| {
                //         Ok(Null::select::<Null>(seed)?.map(|_| Self))
                //     })
                // } else {
                //     Null::select::<S>(seed)
                // }
            },
        )
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
    fn field_name(&self) -> Ident {
        let kind = match self.key {
            SchemaKind::Null => Self::NULL,
            SchemaKind::Bool => Self::BOOL,
            SchemaKind::Int => Self::INT,
            // DataModelKind::Int8 => Self::INT8,
            // DataModelKind::Int16 => Self::INT16,
            // DataModelKind::Int32 => Self::INT32,
            // DataModelKind::Int64 => Self::INT64,
            // DataModelKind::Int128 => Self::INT128,
            // DataModelKind::Uint8 => Self::UINT8,
            // DataModelKind::Uint16 => Self::UINT16,
            // DataModelKind::Uint32 => Self::UINT32,
            // DataModelKind::Uint64 => Self::UINT64,
            // DataModelKind::Uint128 => Self::UINT128,
            SchemaKind::Float => Self::FLOAT,
            // DataModelKind::Float32 => Self::FLOAT32,
            // DataModelKind::Float64 => Self::FLOAT64,
            SchemaKind::Bytes => Self::BYTES,
            SchemaKind::String => Self::STRING,
            SchemaKind::List => Self::LIST,
            SchemaKind::Map => Self::MAP,
            SchemaKind::Link => Self::LINK,
            _ => unreachable!(),
        };

        Ident::new(kind, Span::call_site())
    }

    // fn kind(&self, lib: &TokenStream) -> TokenStream {
    //     match self.key {
    //         DataModelKind::Null => quote! { Kind::Null },
    //         DataModelKind::Boolean => quote! { Kind::Boolean },
    //         DataModelKind::Integer => quote! { Kind::Integer },
    //         DataModelKind::Float => quote! { Kind::Float },
    //         DataModelKind::Bytes => quote! { Kind::Bytes },
    //         DataModelKind::String => quote! { Kind::String },
    //         DataModelKind::List => quote! { Kind::List },
    //         DataModelKind::Map => quote! { Kind::Map },
    //         DataModelKind::Link => quote! { Kind::Link },
    //     }
    // }

    fn field_typedef(&self) -> TokenStream {
        let attrs = &self.attrs;
        let ty = &self.value;
        let field_name = &self.field_name();
        let generics = &self.generics;

        // let implicit_attr = if let Some(implicit) = &self.implicit {
        //     quote!(#[serde(default)])
        // } else {
        //     TokenStream::default()
        // };

        // let rename_attr = if let Some(rename) = &self.rename {
        //     quote!(#[serde(rename = #rename)])
        // } else {
        //     TokenStream::default()
        // };

        let ty = if self.linked {
            quote!(Link<#ty>)
        } else {
            quote!(#ty)
        };

        let branch = if field_name == Self::NULL {
            quote!(#field_name)
        } else {
            quote!(#field_name(#ty #generics))
        };
        quote! {
            #(#attrs)*
            #branch
        }
    }

    fn name_branch(&self, lib: &TokenStream) -> TokenStream {
        let field_name = self.field_name();
        let ty = &self.value;

        if field_name == Self::NULL {
            quote!(Self::#field_name => Null::NAME)
        } else {
            quote!(Self::#field_name(ty) => Representation::name(ty))
        }
    }

    fn kind_branch(&self, lib: &TokenStream) -> TokenStream {
        let field_name = self.field_name();

        if field_name == Self::NULL {
            quote!(Self::#field_name => Null::KIND)
        } else {
            quote!(Self::#field_name(ty) => Representation::kind(ty))
        }
    }
}

impl SchemaKind {}
