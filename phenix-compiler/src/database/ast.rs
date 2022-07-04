use crate::{
    database::vfs::VfsDatabase,
    syntax::{ast, parser::Parse},
    vfs::VfsPath,
};

#[salsa::query_group(AstDatabaseStorage)]
pub trait AstDatabase: VfsDatabase {
    fn parse(&self, path: VfsPath) -> Parse;
    fn ast_root(&self, path: VfsPath) -> ast::Root;
    fn parse_errors(&self, path: VfsPath) -> Vec<String>;
}

fn parse(db: &dyn AstDatabase, path: VfsPath) -> Parse {
    let source = db.read(path);
    crate::syntax::parser::parse(&source)
}

fn ast_root(db: &dyn AstDatabase, path: VfsPath) -> ast::Root {
    db.parse(path).root()
}

fn parse_errors(db: &dyn AstDatabase, path: VfsPath) -> Vec<String> {
    db.parse(path).errors().to_vec()
}
