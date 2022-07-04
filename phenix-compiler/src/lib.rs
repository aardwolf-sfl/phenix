use phenix_codegen::{Module, ModuleId, ModulePath, Project};

mod database;
pub(crate) mod hash;
mod ir;
mod semantics;
mod syntax;
pub mod vfs;

use database::{
    ir::{DefDatabase, DefInternDatabase},
    RootDatabase,
};
use vfs::VfsPath;

pub struct Compiler {
    db: RootDatabase,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            db: RootDatabase::new(),
        }
    }

    pub fn compile(&self, root_file: VfsPath) -> Project {
        let root_file_id = self.db.intern_file(root_file.clone());
        let modules = self.db.reachable_files(root_file_id);

        let root_dir = root_file
            .as_path()
            .parent()
            .expect("vfs path represents a file");

        let modules = modules
            .iter()
            .copied()
            .map(|module_file_id| {
                let module_path = if module_file_id == root_file_id {
                    ModulePath(Vec::new())
                } else {
                    semantics::resolve_module_path(
                        root_dir,
                        self.db.lookup_intern_file(module_file_id).as_path(),
                    )
                };

                let mut types = self
                    .db
                    .module_defs(module_file_id)
                    .values()
                    .copied()
                    .filter_map(|item_id| semantics::make_def(&self.db, module_file_id, item_id))
                    .collect::<Vec<_>>();

                types.sort_by_key(|ty| match ty {
                    phenix_codegen::UserType::Struct(ty) => ty.id,
                    phenix_codegen::UserType::Enum(ty) => ty.id,
                    phenix_codegen::UserType::Flags(ty) => ty.id,
                });

                Module {
                    id: ModuleId(module_file_id.as_usize()),
                    path: module_path,
                    types,
                }
            })
            .collect();

        Project { modules }
    }

    pub fn compile_and_watch<F>(&mut self, root_file: VfsPath, mut callback: F)
    where
        F: FnMut(Project),
    {
        loop {
            callback(self.compile(root_file.clone()));

            if !self.db.watch() {
                break;
            }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
