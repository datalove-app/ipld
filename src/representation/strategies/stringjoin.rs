use crate::dev::*;

const STRATEGY: Strategy = Strategy::StringJoin;

/*
// Blanket impl for structs.
macro_rules! stringjoin {
    () => {
        repr_serde! { @visitors for T => (K, V)
            { @dk (type_kinds::Map) @sk (type_kinds::Map) @rk (type_kinds::String) }
            { T, K, V } { T: Default + Extend<(K, V)> +  'static,
                          K: Select<Ctx> + StringRepresentation + 'static,
                          <K as FromStr>::Err: fmt::Display,
                          V: Select<Ctx> + 'static } @serde {
            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A list of type {} of {}", S::NAME, T::NAME)
            }
        }
    }
}
 */
