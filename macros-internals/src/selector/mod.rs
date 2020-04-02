//! `selector!` macro.
//!
//! Outputs a statically-typed instance of `Selector`, checked against the schemas of the input type (as well as any nested types), and the desired output type.
//!
//! # Syntax:
//! ```edition2018
//! /// Selects the 'parent' of a blockchain block.
//! let ParentSelector = selector!(BlockchainBlock,
//!     fields('parent'(
//!         match
//!     ))
//! )
//! ```
//!
/// TODO? impl should look roughly like:
///
/// let type: Type = input.parse()?;
/// let fields & field_name = ...;
/// let rest: TokenStream = ...;
/// quote! {{
///     let mut explore_fields = ExploreFields::new();
///
///     *(
///         let sel = <BlockchainBlock as Select<ExploreField<#field_name>>>
///             ::insert(
///                 explore_fields as ExploreFields,
///                 #match_args...?
///             );
///
///         explore_fields = sel.0;
///     );
///
///     Selector::ExploreFields()
/// }}
///
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream, Result as ParseResult};

pub struct SelectorDefinition;

impl SelectorDefinition {
    pub fn expand(self) -> TokenStream {
        self.into_token_stream()
    }
}

impl Parse for SelectorDefinition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self)
    }
}

impl ToTokens for SelectorDefinition {
    /// TODO?
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(TokenStream::default());
    }
}

pub(crate) mod kw {
    crate::define_keywords! {
        fields
        recurse recursive
        all
        limit
        stopAt
        label
        union
    }
}

/* Examples:

recursive(limit=5
  fields(
    'tree'(
      recursive(
        all(recurse)
      )
    )
    'parents'(
      all(recurse)
    )
  )
)

# Starting at the commit block.
R5f'tree'R*~'parents'*~

fields('characters'(
  fields('kathryn-janeway'(
    fields('birthday'(
      fields('year'(match))
    ))
  ))
))

f'characters'f'kathryn-janeway'f'birthday'f'year'.

# Long Form
fields('parent'(
  fields('parent'(
    fields('parent'(
      fields('parent'(
        fields('parent'(
          match
        ))
      ))
    ))
  ))
))

# Short Form
f'parent'f'parent'f'parent'f'parent'f'parent'.

# Long Form
recursive(limit=5
  fields('parent'(
    recurse
  ))
)

# Short Form
R5f'parent'~

# Long Form
recursive(
  limit=100
  fields(
    'prev'(recurse)
  )
  #stopAt=... # Conditions are not specified yet
)

# Short Form
R100f'prev'~#... # Conditions are not specified yet

# Long Form
recursive(limit=1000
  fields(
    'data'(match)
    'links'(
      all(
        fields('cid'(
          recurse
        ))
      )
    )
  )
)

# Short Form
R1000f'data'.()'links'*f'cid'~

## The following examples are aimed more at exercising the parser.

# String with escaped quote embedded
f'What\'s up'.

# String with newlines embedded
f'this
has
newlines'.

# Match with index
i5.

# Match with union
u(
  i5.
  f'stuff'.
)

# Nested unions
union(
  union(
    match
  )
  match
)

# Proper short form with parentheses
uu(.).

# Broken short form that changes semantics
uu..

# Fields with labels, maximum verbosity mode
fields(
  fields=(
    'with-label'(
      match(
        label=('label')
      )
    )
    'without-label'(
      match()
    )
  )
)

# Fields with labels, human readable mode
fields
  'with-label'
    match(label='label')
  'without-label'
    match

# Properly Minimized
f'with-label'(.'label')'without-label'.

# Another Properly Minimized
f'with-label'.('label')'without-label'.

# Another valid form
f'with-label'.label=('label')'without-label'.


fields
  'foo'
    match
      label='blue'
  'bar'
    match

# valid with new string encoding
f'with-label'.'label''without-label'.

.'This is a string isn\'t it?'

.'This is also a "string".'

.'This has\nescapes\tthat work.'

.'Lots-o-escapes "\\\b\f\n\r\t"'

# Embedded unicode support
.'😷'

.'More escapes \'with quotes\''

# This should fail with "Invalid escape at 17"
.'Invalid escape \"'

# This should fail with "Unterminated string at 1"
.'unclosed string

 */
