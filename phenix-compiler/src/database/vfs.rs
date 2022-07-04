use std::sync::Arc;

use crate::vfs::{VfsPath, VfsWatcher};

#[salsa::query_group(VfsDatabaseStorage)]
pub trait VfsDatabase: salsa::Database + VfsWatcher {
    fn read(&self, path: VfsPath) -> Arc<String>;
}

fn read(db: &dyn VfsDatabase, path: VfsPath) -> Arc<String> {
    db.salsa_runtime()
        .report_synthetic_read(salsa::Durability::LOW);
    db.watch(&path);
    Arc::new(std::fs::read_to_string(path.as_path()).unwrap_or_default())
}

pub fn invalidate_path(db: &mut dyn VfsDatabase, path: &VfsPath) {
    ReadQuery.in_db_mut(db).invalidate(path)
}
