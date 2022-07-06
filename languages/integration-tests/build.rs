use std::io;

fn main() -> io::Result<()> {
    phenix_build::compile("schema/index.phenix")
}
