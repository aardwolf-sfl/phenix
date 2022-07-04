use crate::vfs::VfsWatcher;

pub mod ast;
pub mod ir;
pub mod vfs;

#[salsa::database(
    vfs::VfsDatabaseStorage,
    ast::AstDatabaseStorage,
    ir::DefInternDatabaseStorage,
    ir::DefDatabaseStorage
)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
    vfs: crate::vfs::Vfs,
}

impl salsa::Database for RootDatabase {}

impl RootDatabase {
    pub fn new() -> Self {
        Self {
            storage: Default::default(),
            vfs: crate::vfs::Vfs::new(),
        }
    }

    pub fn watch(&mut self) -> bool {
        match self.vfs.recv() {
            Ok((path, kind)) => {
                self.on_change(&path, kind);
                true
            }
            Err(_) => false,
        }
    }
}

impl Default for RootDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::vfs::VfsWatcher for RootDatabase {
    fn watch(&self, path: &crate::vfs::VfsPath) {
        self.vfs.watch(path);
    }

    fn on_change(&mut self, path: &crate::vfs::VfsPath, _: crate::vfs::ChangeKind) {
        vfs::invalidate_path(self, path)
    }
}
