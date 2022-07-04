use derive_more::{Deref, From, Into};
use serde::{Deserialize, Serialize};

pub(crate) mod shared;

mod rust;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Language {
    Rust,
}

impl Language {
    pub fn extension(&self) -> &str {
        match self {
            Language::Rust => "rs",
        }
    }
}

pub fn generate(project: Project, lang: Language) -> String {
    let ctx = shared::ProjectContext::new(project);

    match lang {
        Language::Rust => rust::generate(ctx),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BuiltinType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Uint,
    Sint,
    Float,
    String,
    Vector,
    Stream,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize,
    Serialize,
    From,
    Into,
    Deref,
)]
#[repr(transparent)]
pub struct UserTypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, From)]
#[serde(rename_all = "snake_case")]
pub enum TypeId {
    Builtin(BuiltinType),
    User(UserTypeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Type {
    pub id: TypeId,
    pub generics: Vec<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Attribute {
    NonExhaustive,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct StructType {
    pub id: UserTypeId,
    pub name: String,
    pub fields: Vec<Field>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct EnumType {
    pub id: UserTypeId,
    pub name: String,
    pub variants: Vec<Variant>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FlagsType {
    pub id: UserTypeId,
    pub name: String,
    pub flags: Vec<String>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, From)]
#[serde(tag = "type")]
pub enum UserType {
    Struct(StructType),
    Enum(EnumType),
    Flags(FlagsType),
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize,
    Serialize,
    From,
    Into,
    Deref,
)]
#[repr(transparent)]
pub struct ModuleId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, From, Into, Deref)]
pub struct ModulePath(pub Vec<String>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Module {
    pub id: ModuleId,
    pub path: ModulePath,
    pub types: Vec<UserType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Project {
    pub modules: Vec<Module>,
}
