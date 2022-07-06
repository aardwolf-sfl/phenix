use std::{
    fs, io,
    path::{Path, PathBuf},
};

use phenix_codegen::Language;

#[derive(Debug, Default, Clone)]
pub struct Config {
    out_dir: Option<PathBuf>,
    out_file: Option<String>,
}

impl Config {
    pub fn out_dir<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.out_dir = Some(path.into());
        self
    }

    pub fn out_file<P: Into<String>>(&mut self, name: P) -> &mut Self {
        self.out_file = Some(name.into());
        self
    }

    pub fn compile<P: AsRef<Path>>(&self, root_file: P) -> io::Result<()> {
        compile_with_config(root_file, self.clone())
    }
}

pub fn compile<P: AsRef<Path>>(root_file: P) -> io::Result<()> {
    compile_with_config(root_file, Config::default())
}

fn compile_with_config<P: AsRef<Path>>(root_file: P, config: Config) -> io::Result<()> {
    let root_file = root_file.as_ref();

    let out_dir = config.out_dir.map(Ok).unwrap_or_else(|| {
        std::env::var_os("OUT_DIR")
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "OUT_DIR environment variable is not set",
                )
            })
            .map(Into::into)
    })?;

    let out_file = config.out_file.as_deref().unwrap_or_else(|| {
        root_file
            .file_name()
            .and_then(|file| file.to_str())
            .unwrap_or("generated")
    });
    let mut out_file = PathBuf::from(out_file);
    out_file.set_extension("rs");

    let project = phenix_compiler::Compiler::new().compile(root_file.try_into()?);
    let generated = phenix_codegen::generate(project, Language::Rust);

    let out_file = out_dir.join(out_file);
    fs::write(out_file, generated)
}
