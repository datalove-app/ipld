use super::*;
use crate::dev::schema::{
    expand::{impl_repr, ExpandBasicRepresentation},
    SchemaKind, SchemaMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for KindedUnionReprDefinition {
    fn define_type(&self, meta: &SchemaMeta) -> TokenStream {
        let attrs = &meta.attrs;
        let vis = &meta.vis;
        let name = &meta.name;
        let fields = self.iter().map(UnionKindedField::typedef);

        // TODO: assert that all nested REPR_KINDs are unique and are DM kinds

        quote! {
            #(#attrs)*
            // #[derive(Deserialize, Serialize)]
            // #[serde(untagged)]
            #vis enum #name {
                #(#fields,)*
            }
        }
    }

    fn derive_repr(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;
        let dm_ty = self.dm_ty();
        // let repr_kind = self.repr_kind();
        let name_branches = self.iter().map(UnionKindedField::name_branch);
        // let kind_branches: Vec<TokenStream> = self.iter().map(|f| f.kind_branch(&lib)).collect();
        let serialize_branches = self.iter().map(UnionKindedField::serialize_branch);
        // let deserialize_branches = self.iter().map(UnionKindedField::deserialize_branch);

        let non_link_visitors = self.visitors(meta, false, false);
        let link_visitors = self.visitors(meta, true, false);
        impl_repr(
            meta,
            quote! {
                type DataModelKind = #dm_ty;
                type SchemaKind = type_kinds::Union;
                // FIXME:
                // cannot be the literal union of the types, as that
                // confuses the type checker
                type ReprKind = type_kinds::Any;

                const SCHEMA: &'static str = "";
                const DATA_MODEL_KIND: Kind = <#dm_ty>::KIND;
                const SCHEMA_KIND: Kind = Kind::Union;
                const REPR_KIND: Kind = type_kinds::Any::KIND;
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

                #[doc(hidden)]
                fn serialize<const C: u64, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    match self {
                        #(#serialize_branches,)*
                    }
                }

                #[doc(hidden)]
                fn deserialize<'de, const C: u64, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct KindedVisitor<const C: u64>;
                    impl<'de, const C: u64> Visitor<'de> for KindedVisitor<C> {
                        type Value = #name;
                        #[inline]
                        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                            write!(f, "{}", stringify!(#name))
                        }
                        #(#non_link_visitors)*
                    }
                    impl<'de, const C: u64> LinkVisitor<'de> for KindedVisitor<C> {
                        #(#link_visitors)*
                    }

                    Multicodec::deserialize_any::<C, _, _>(deserializer, KindedVisitor::<C>)
                }
            },
        )
    }

    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let lib = &meta.lib;
        let name = &meta.name;
        let dm_ty = self.dm_ty();

        let non_link_visitors = self.visitors(meta, false, true);
        let link_visitors = self.visitors(meta, true, true);

        // TODO: add a method that does the delegation to the variant, rather than codegen each
        quote! {
            #lib::dev::macros::repr_serde! { @visitors for #name => #name
                { @dk (#dm_ty) @sk (type_kinds::Union) @rk (type_kinds::Any) }
                {} {}
                @serde {
                    #[inline]
                    fn expecting(&self, f: &mut maybestd::fmt::Formatter<'_>) -> maybestd::fmt::Result {
                        write!(f, "A `{}`", <#name as Representation>::NAME)
                    }
                    #(#non_link_visitors)*
                }
                @link {
                    #(#link_visitors)*
                }
            }

            #lib::dev::macros::repr_serde! { @select_for #name => #name
                { @dk (#dm_ty) @sk (type_kinds::Union) @rk (type_kinds::Any) }
                {} {}
            }
        }
    }

    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}

impl KindedUnionReprDefinition {
    // TODO: should be the union of all the actual type's data models (since they may not be the standard)
    // fn dm_kind(&self) -> TokenStream {
    //     self.iter()
    //         .map(|f| f.dm_kind())
    //         .fold(quote!(Kind::empty()), |ts, kind| quote!(#ts.union(#kind)))
    // }

    fn dm_ty(&self) -> TokenStream {
        self.iter().map(|f| f.ty(true)).fold(
            quote!(type_kinds::Empty),
            |ts, ty| quote!(typenum::Or<#ts, <#ty as Representation>::DataModelKind>),
        )
    }

    fn visitors<'a>(
        &'a self,
        meta: &'a SchemaMeta,
        for_link: bool,
        for_seed: bool,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        self.iter()
            .filter(move |f| {
                if for_link {
                    f.key == SchemaKind::Link
                } else {
                    f.key != SchemaKind::Link
                }
            })
            .map(move |f| f.visit_fn(meta, for_seed))
            .flatten()
    }
}

impl UnionKindedField {
    /// Outputs the kinded enum variant name.
    /// TODO: this is wrong, let users define the name and just check the repr_kind
    fn name(&self) -> &Ident {
        &self.value
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

    fn serialize_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::serialize::<C, _>(ty, serializer))
    }
}

impl UnionKindedField {
    fn visit_parts(&self) -> Vec<(TokenStream, TokenStream)> {
        match self.key {
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
                (quote!(visit_borrowed_bytes), quote!(v: &'de [u8])),
                (quote!(visit_bytes), quote!(v: &[u8])),
                (quote!(visit_byte_buf), quote!(v: Vec<u8>)),
            ],
            SchemaKind::String => vec![
                (quote!(visit_borrowed_str), quote!(v: &'de str)),
                (quote!(visit_str), quote!(v: &str)),
                (quote!(visit_string), quote!(v: String)),
            ],
            SchemaKind::List => vec![(quote!(visit_seq), quote!(v: A))],
            SchemaKind::Map => vec![(quote!(visit_map), quote!(v: A))],
            SchemaKind::Link => vec![
                (quote!(visit_link_borrowed_bytes), quote!(v: &'de [u8])),
                (quote!(visit_link_bytes), quote!(v: &[u8])),
                (quote!(visit_link_borrowed_str), quote!(v: &'de str)),
                (quote!(visit_link_str), quote!(v: &str)),
                // TODO: provide an impl for this
                // (quote!(visit_link), quote!(v: Cid)),
            ],
            _ => unreachable!(),
        }
    }

    // TODO: potentially inefficient handling of borrowed/owned strs and bytes
    fn visit_fn(
        &self,
        meta: &SchemaMeta,
        for_seed: bool,
    ) -> impl Iterator<Item = TokenStream> + '_ {
        let name = &meta.name;
        let field_name = self.name();
        let ty = self.ty(false);
        let visit_impl = {
            // TODO: match key's REPR_KIND
            let deserializer = match self.key {
                SchemaKind::Null => quote!(().into_deserializer()),
                SchemaKind::String => quote!(serde::de::value::CowStrDeserializer::new(v.into())),
                SchemaKind::Bytes => quote!(serde::de::value::BytesDeserializer::new(&v)),
                SchemaKind::List => quote!(serde::de::value::SeqAccessDeserializer::<A>::new(v)),
                SchemaKind::Map => quote!(serde::de::value::MapAccessDeserializer::<A>::new(v)),
                _ => quote!(v.into_deserializer()),
            };

            if for_seed {
                quote! {
                    let seed = self.0
                        .wrap::<#ty, _>(|dag| #name::#field_name(dag.into()));
                    <#ty>::__select_de::<C, _>(seed, #deserializer)
                }
            } else {
                quote! {
                    let inner = <#ty as Representation>::deserialize::<C, _>(#deserializer)?;
                    Ok(#name::#field_name(inner.into()))
                }
            }
        };

        self.visit_parts()
            .into_iter()
            .map(move |(visit_fn, args)| match self.key {
                SchemaKind::List => quote! {
                    #[inline]
                    fn #visit_fn<A>(mut self, #args) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>
                    {
                        #visit_impl
                    }
                },
                SchemaKind::Map => quote! {
                    #[inline]
                    fn #visit_fn<A>(mut self, #args) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>
                    {
                        #visit_impl
                    }
                },
                _ => quote! {
                    #[inline]
                    fn #visit_fn<E>(mut self, #args) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        #visit_impl
                    }
                },
            })
    }
}
