use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use clap::{clap_derive::ArgEnum, Parser};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, arg_enum, value_parser)]
    language: Language,

    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,

    #[clap(value_parser)]
    input: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
enum Language {
    Rust,
    C,
}

impl From<Language> for phenix_codegen::Language {
    fn from(lang: Language) -> Self {
        match lang {
            Language::Rust => phenix_codegen::Language::Rust,
            Language::C => phenix_codegen::Language::C,
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let input = phenix_compiler::vfs::VfsPath::new(args.input)?;
    let project = phenix_compiler::Compiler::new().compile(input);
    let generated = phenix_codegen::generate(project, args.language.into());

    match args.output {
        Some(output) => fs::write(output, generated),
        None => io::stdout().lock().write_all(generated.as_bytes()),
    }
}
