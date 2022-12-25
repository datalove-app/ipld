use super::*;
use crate::dev::schema::{expand::ExpandBasicRepresentation, SchemaKind, SchemaMeta};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

impl ExpandBasicRepresentation for KindedUnionReprDefinition {
    fn schema(&self, meta: &SchemaMeta) -> TokenStream {
        // const SCHEMA: &'static str = concat!(
        //     "type ", #name_str, " union {",
        //     "}"
        // );
        Default::default()
    }

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
        let dm_kind = self.dm_kind();

        let names = self.iter().map(UnionKindedField::name_branch);
        let as_fields = self.iter().map(UnionKindedField::as_field_branch);
        let to_selected_nodes = self.iter().map(UnionKindedField::to_selected_node_branch);
        let serializes = self.iter().map(UnionKindedField::serialize_branch);

        let expecting = self.expecting(meta);
        let non_link_visitors = self.visitors(meta, false, false);
        let link_visitors = self.visitors(meta, true, false);

        let consts = quote! {
            const DATA_MODEL_KIND: Kind = #dm_kind;
            const SCHEMA_KIND: Kind = Kind::Union;
            const REPR_KIND: Kind = Kind::Any;
            const REPR_STRATEGY: Strategy = Strategy::Kinded;
            // TODO
            // const FIELDS: Fields = Fields::Keyed(&[#(#fields,)*]);
        };
        self.impl_repr(
            meta,
            consts,
            quote! {
                #[inline]
                fn name(&self) -> &'static str {
                    match self {
                        #(#names,)*
                    }
                }
                #[inline]
                fn as_field(&self) -> Option<Field<'_>> {
                    match self {
                        #(#as_fields,)*
                    }
                }
                #[inline]
                fn to_selected_node(&self) -> SelectedNode {
                    match self {
                        #(#to_selected_nodes,)*
                    }
                }
                #[inline]
                fn serialize<const MC: u64, Se: Serializer>(
                    &self, serializer: Se
                ) -> Result<Se::Ok, Se::Error> {
                    match self {
                        #(#serializes,)*
                    }
                }
                #[inline]
                fn deserialize<'de, const MC: u64, De>(de: De) -> Result<Self, De::Error>
                where
                    De: Deserializer<'de>
                {
                    struct V<const MC: u64>;
                    impl<'de, const MC: u64> Visitor<'de> for V<MC> {
                        type Value = AstResult<#name>;

                        #expecting

                        #(#non_link_visitors)*
                    }
                    impl<'de, const MC: u64> LinkVisitor<'de, MC> for V<MC> {
                        #(#link_visitors)*
                    }

                    let res = Multicodec::deserialize_any::<MC, De, _>(de, V::<MC>)?;
                    Ok(res.unwrap_val())
                }
            },
        )
    }

    fn derive_select(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name;

        let expecting = self.expecting(meta);
        let non_link_visitors = self.visitors(meta, false, true);
        let link_visitors = self.visitors(meta, true, true);

        let select = self.impl_select(meta, None);
        quote! {
            #select
            repr_serde!(@visitors for #name where @serde {
                #expecting
                #(#non_link_visitors)*
            } @link {
                #(#link_visitors)*
            });
        }
    }

    fn derive_conv(&self, meta: &SchemaMeta) -> TokenStream {
        quote!()
    }
}

impl KindedUnionReprDefinition {
    fn dm_kind(&self) -> TokenStream {
        self.iter().map(|f| f.ty(true)).fold(
            quote!(Kind::empty()),
            |ts, ty| quote!(#ts.union(<#ty as Representation>::DATA_MODEL_KIND)),
        )
    }

    fn expecting(&self, meta: &SchemaMeta) -> TokenStream {
        let name = &meta.name.to_string();
        quote! {
            #[inline]
            fn expecting(&self, f: &mut maybestd::fmt::Formatter<'_>) -> maybestd::fmt::Result {
                write!(f, "A `{}`", #name)
            }
        }
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

    fn ty(&self, with_wrapper: bool) -> TokenStream {
        let name = self.name();
        let generics = &self.generics;

        self.linked
            .then(|| quote!(Link<#name #generics>))
            .or_else(|| Some(quote!(#name #generics)))
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
        quote!(Self::#name(ty) => Representation::name(ty))
    }
    fn as_field_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::as_field(ty))
    }
    fn to_selected_node_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::to_selected_node(ty))
    }
    fn serialize_branch(&self) -> TokenStream {
        let name = self.name();
        quote!(Self::#name(ty) => Representation::serialize::<MC, Se>(ty, serializer))
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
        let variant = self.name();
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
                    let seed = self
                        .into_inner()
                        .wrap::<#ty, _>(|dag: #ty| #name::#variant(dag.into()));

                    <#ty>::__select_de::<MC, _>(seed, #deserializer)
                }
            } else {
                quote! {
                    let dag = <#ty as Representation>::deserialize::<MC, _>(#deserializer)?;
                    Ok(#name::#variant(dag.into()))
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
                        // { #visit_impl }?;
                        Ok(AstResult::Ok)
                    }
                },
                SchemaKind::Map => quote! {
                    #[inline]
                    fn #visit_fn<A>(mut self, #args) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::MapAccess<'de>
                    {
                        // { #visit_impl }?;
                        Ok(AstResult::Ok)
                    }
                },
                _ => quote! {
                    #[inline]
                    fn #visit_fn<E>(mut self, #args) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error
                    {
                        // { #visit_impl }?;
                        Ok(AstResult::Ok)
                    }
                },
            })
    }
}
