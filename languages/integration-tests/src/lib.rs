#![cfg(test)]

use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    process::{Command, Stdio},
};

use phenix_runtime::{bytes::Bytes, prelude::*, Stream};
use tempfile::NamedTempFile;

mod schema {
    include!(concat!(env!("OUT_DIR"), "/index.rs"));
}

fn expected() -> (schema::Person, Vec<schema::Project>) {
    let person = schema::Person {
        name: "Felix".to_string(),
        age: 42.into(),
        pronouns: schema::Pronouns {
            subject: schema::nested::Pronoun::They,
            object: schema::nested::Pronoun::Them,
        },
        degree: schema::Degree::Highest {
            name: schema::nested::DegreeName::Master,
        },
        citizenship: schema::nested::Country::default()
            .set(schema::nested::CountryFlag::CzechRepublic)
            .set(schema::nested::CountryFlag::France)
            .clone(),
        working_hours: vec![
            false, false, false, false, false, false, false, false, false, true, true, true, true,
            true, true, true, true, false, false, false, false, false, false, false,
        ],
        projects: Stream::with_offset(19),
    };

    let projects = vec![
        schema::Project {
            name: "Rust".to_string(),
            url: "https://github.com/rust-lang/rust".to_string(),
        },
        schema::Project {
            name: "Linux".to_string(),
            url: "https://github.com/torvalds/linux".to_string(),
        },
        schema::Project {
            name: "Phenix".to_string(),
            url: "https://github.com/aardwolf-sfl/phenix".to_string(),
        },
    ];

    (person, projects)
}

#[test]
fn rust_to_rust() {
    let (person, projects) = expected();

    let mut cursor = Cursor::new(Vec::new());

    person.encode(&mut cursor).unwrap();

    for project in projects.iter() {
        Stream::push_encode(project, &mut cursor).unwrap();
    }

    let bytes = cursor.into_inner();

    let decoded = schema::Person::decode(&mut Bytes::new(&bytes), &mut Vec::new()).unwrap();
    let collected = decoded.projects.collect(&bytes).unwrap();

    assert_eq!(decoded, person);
    assert_eq!(collected, projects);
}

#[test]
fn c_to_rust() {
    let c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("c");

    run("make", |command| command.arg("clean").current_dir(&c_dir));
    run("make", |command| {
        command.arg("generate-phenix").current_dir(&c_dir)
    });
    run("make", |command| command.arg("build").current_dir(&c_dir));

    let tmp = NamedTempFile::new().unwrap();

    run("./run", |command| {
        command.arg(tmp.path()).current_dir(&c_dir)
    });

    let bytes = fs::read(tmp.path()).unwrap();

    let decoded = schema::Person::decode(&mut Bytes::new(&bytes), &mut Vec::new()).unwrap();
    let collected = decoded.projects.collect(&bytes).unwrap();

    let (person, projects) = expected();

    assert_eq!(decoded, person);
    assert_eq!(collected, projects);
}

fn run<S: AsRef<std::ffi::OsStr>, F>(command: S, builder: F)
where
    F: FnOnce(&mut Command) -> &mut Command,
{
    let mut command = Command::new(command);
    builder(&mut command);

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    // Print the output unconditionally, because it may be useful for diagnosing
    // a problem that happened while running a later command.
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}
