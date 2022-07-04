pub type FxIndexSet<T> = indexmap::IndexSet<T, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
