use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use crate::{hash::FxIndexSet, ir::*, vfs::VfsPath};

use super::ast::AstDatabase;

macro_rules! def_intern_key {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(salsa::InternId);

        impl salsa::InternKey for $name {
            fn from_intern_id(id: salsa::InternId) -> Self {
                $name(id)
            }

            fn as_intern_id(&self) -> salsa::InternId {
                self.0
            }
        }
    };
}

def_intern_key!(VfsFileId);
def_intern_key!(ItemId);

impl VfsFileId {
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }
}

impl ItemId {
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemLoc<T> {
    file: VfsFileId,
    item: T,
}

impl<T> ItemLoc<T> {
    pub fn new(file: VfsFileId, item: T) -> Self {
        Self { file, item }
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}

impl<T> std::ops::Deref for ItemLoc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[salsa::query_group(DefInternDatabaseStorage)]
pub trait DefInternDatabase {
    #[salsa::interned]
    fn intern_file(&self, value: VfsPath) -> VfsFileId;

    #[salsa::interned]
    fn intern_item(&self, value: ItemLoc<ItemData>) -> ItemId;
}

#[salsa::query_group(DefDatabaseStorage)]
pub trait DefDatabase: DefInternDatabase + AstDatabase {
    fn reachable_files(&self, root_file: VfsFileId) -> Arc<FxIndexSet<VfsFileId>>;
    fn module_defs(&self, file: VfsFileId) -> Arc<FxHashMap<String, ItemId>>;
    fn module_imports(&self, file: VfsFileId) -> Arc<FxHashMap<String, ItemId>>;
    fn module_scope(&self, file: VfsFileId) -> Arc<FxHashMap<String, ItemId>>;
    fn module_item_by_name(&self, file: VfsFileId, name: String) -> Option<ItemId>;
}

fn reachable_files(db: &dyn DefDatabase, root_file: VfsFileId) -> Arc<FxIndexSet<VfsFileId>> {
    let path = db.lookup_intern_file(root_file);
    let importer = Importer::new(&path);

    let root = db.ast_root(path);

    let reachable = std::iter::once(root_file)
        .chain(
            root.imports()
                .filter_map(|import| {
                    let import_file = db.intern_file(importer.import_path(import.path()?)?);
                    let recursive = db.reachable_files(import_file);

                    Some(recursive.iter().copied().collect::<Vec<_>>())
                })
                .flatten(),
        )
        .collect();

    Arc::new(reachable)
}

fn module_defs(db: &dyn DefDatabase, file: VfsFileId) -> Arc<FxHashMap<String, ItemId>> {
    let path = db.lookup_intern_file(file);
    let root = db.ast_root(path);

    let defs = root
        .defs()
        .filter_map(ItemData::from_ast)
        .map(|item| {
            let name = item.name().to_string();
            let data = match item {
                ItemData::Struct(item) => ItemLoc::new(file, ItemData::Struct(item)),
                ItemData::Enum(item) => ItemLoc::new(file, ItemData::Enum(item)),
                ItemData::Flags(item) => ItemLoc::new(file, ItemData::Flags(item)),
            };
            let id = db.intern_item(data);
            (name, id)
        })
        .collect();

    Arc::new(defs)
}

fn module_imports(db: &dyn DefDatabase, file: VfsFileId) -> Arc<FxHashMap<String, ItemId>> {
    let path = db.lookup_intern_file(file);
    let importer = Importer::new(&path);

    let root = db.ast_root(path);

    let imports = root
        .imports()
        .filter_map(|import| {
            let import_file = db.intern_file(importer.import_path(import.path()?)?);

            let imports: Vec<_> = if import.is_star() {
                db.module_scope(import_file)
                    .iter()
                    .map(|(name, id)| (name.clone(), *id))
                    .collect()
            } else {
                import
                    .aliases()
                    .filter_map(move |alias| {
                        let from = alias.name_from()?.to_string();
                        let to = alias.name_to()?.to_string();

                        let def_id = *db.module_scope(import_file).get(&from)?;

                        Some((to, def_id))
                    })
                    .collect()
            };

            Some(imports)
        })
        .flatten()
        .collect();

    Arc::new(imports)
}

fn module_scope(db: &dyn DefDatabase, file: VfsFileId) -> Arc<FxHashMap<String, ItemId>> {
    let scope = db
        .module_imports(file)
        .iter()
        .chain(db.module_defs(file).iter())
        .map(|(name, id)| (name.clone(), *id))
        .collect();

    Arc::new(scope)
}

fn module_item_by_name(db: &dyn DefDatabase, file: VfsFileId, name: String) -> Option<ItemId> {
    db.module_scope(file).get(&name).copied()
}

struct Importer {
    base: PathBuf,
}

impl Importer {
    fn new(path: &VfsPath) -> Self {
        Self {
            base: path
                .as_path()
                .parent()
                .expect("vfs path represents a file")
                .to_path_buf(),
        }
    }

    fn import_path<P: AsRef<Path>>(&self, import_path: P) -> Option<VfsPath> {
        VfsPath::new(self.base.join(import_path.as_ref())).ok()
    }
}
