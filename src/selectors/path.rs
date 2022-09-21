use crate::dev::*;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

// ///
// #[derive(Debug)]
// pub struct PathSeed<T> {
//     max_block_depth: usize,
//     path: PathBuf,
//     _type: PhantomData<T>,
// }

// impl<'de, T: Representation> DeserializeSeed<'de> for PathSeed<T> {
//     type Value = (Option<Self>, Option<T>);

//     #[inline]
//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         T::try_deserialize_path(self, deserializer).map_err(D::Error::custom)
//     }
// }

impl From<Path> for Selector {
    fn from(path: Path) -> Self {
        unimplemented!()
    }
}
