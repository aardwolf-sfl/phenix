use std::{env, fmt, fs, io, path::Path};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    generate_primitives("src/generated.rs").unwrap();

    std::process::Command::new("rustfmt")
        .args(&["--edition", "2021", "src/generated.rs"])
        .spawn()
        .unwrap();

    cbindgen::generate(crate_dir)
        .unwrap()
        .write_to_file("include/phenix_runtime.h");
}

fn generate_primitives<P: AsRef<Path>>(path: P) -> io::Result<()> {
    struct Primitive {
        name: &'static str,
        ty: &'static str,
        transmute: Option<&'static str>,
    }

    impl Primitive {
        const fn new(
            name: &'static str,
            ty: &'static str,
            transmute: Option<&'static str>,
        ) -> Self {
            Self {
                name,
                ty,
                transmute,
            }
        }
    }

    impl fmt::Display for Primitive {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // encode
            writeln!(f, "#[no_mangle]")?;
            write!(f, "pub extern \"C\" fn ")?;
            write!(f, "phenix_runtime_{}_encode", self.name)?;
            write!(
                f,
                "(value: {}, stream: *mut libc::FILE) -> libc::c_int",
                self.ty
            )?;
            writeln!(f, " {{")?;
            match self.transmute {
                Some(transmute) => writeln!(
                    f,
                    "crate::call_encode(&<{}>::from(value), stream)",
                    transmute
                )?,
                None => writeln!(f, "crate::call_encode(&value, stream)",)?,
            }
            writeln!(f, "}}")?;
            writeln!(f)?;

            // encode many
            writeln!(f, "#[no_mangle]")?;
            write!(f, "pub extern \"C\" fn ")?;
            write!(f, "phenix_runtime_{}_encode_many", self.name)?;
            write!(
                f,
                "(values: *const {}, n: usize, stream: *mut libc::FILE) -> libc::c_int",
                self.ty
            )?;
            writeln!(f, " {{")?;
            match self.transmute {
                Some(transmute) => writeln!(
                    f,
                    "crate::call_encode_many(values.cast::<{}>(), n, stream)",
                    transmute
                )?,
                None => writeln!(f, "crate::call_encode_many(values, n, stream)",)?,
            }
            writeln!(f, "}}")?;
            writeln!(f)?;

            Ok(())
        }
    }

    let primitives = [
        Primitive::new("uint", "u64", Some("phenix_runtime::Uint")),
        Primitive::new("sint", "i64", Some("phenix_runtime::Sint")),
        Primitive::new("float", "f64", Some("phenix_runtime::Float")),
        Primitive::new("bool", "bool", None),
        Primitive::new("u8", "u8", None),
        Primitive::new("u16", "u16", None),
        Primitive::new("u32", "u32", None),
        Primitive::new("u64", "u64", None),
        Primitive::new("i8", "i8", None),
        Primitive::new("i16", "i16", None),
        Primitive::new("i32", "i32", None),
        Primitive::new("i64", "i64", None),
        Primitive::new("f32", "f32", None),
        Primitive::new("f64", "f64", None),
    ];

    let mut generated = String::new();
    for primitive in primitives {
        generated.push_str(&format!("{}", primitive));
    }

    fs::write(path, generated)
}
