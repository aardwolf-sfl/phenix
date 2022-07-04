use std::path::Path;

use phenix_codegen::Project;
use phenix_compiler::Compiler;

fn compile<P: AsRef<Path>>(path: P) -> Project {
    Compiler::new().compile(path.as_ref().try_into().unwrap())
}

#[test]
fn basic() {
    let project = compile("tests/schemas/basic/index.phenix");
    insta::assert_yaml_snapshot!(project);
}

#[test]
fn imports() {
    let project = compile("tests/schemas/imports/index.phenix");
    insta::assert_yaml_snapshot!(project);
}
