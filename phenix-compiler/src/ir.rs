use derive_more::From;

use crate::syntax::ast::{self, HasName};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructData {
    pub name: String,
    pub fields: Vec<FieldData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumData {
    pub name: String,
    pub variants: Vec<VariantData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagsData {
    pub name: String,
    pub flags: Vec<FlagData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub enum ItemData {
    Struct(StructData),
    Enum(EnumData),
    Flags(FlagsData),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldData {
    pub name: String,
    pub ty: TypeData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantData {
    pub name: String,
    pub fields: Vec<FieldData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagData {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeData {
    pub name: String,
    pub generics: Vec<TypeData>,
}

impl ItemData {
    pub fn from_ast(node: ast::ItemDef) -> Option<Self> {
        match node.kind() {
            ast::ItemDefKind::Struct(item) => StructData::from_ast(item).map(ItemData::Struct),
            ast::ItemDefKind::Enum(item) => EnumData::from_ast(item).map(ItemData::Enum),
            ast::ItemDefKind::Flags(item) => FlagsData::from_ast(item).map(ItemData::Flags),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ItemData::Struct(data) => data.name.as_str(),
            ItemData::Enum(data) => data.name.as_str(),
            ItemData::Flags(data) => data.name.as_str(),
        }
    }
}

impl StructData {
    pub fn from_ast(node: ast::StructDef) -> Option<Self> {
        let name = node.name()?.to_string();
        let fields = node.fields().filter_map(FieldData::from_ast).collect();

        Some(StructData { name, fields })
    }
}

impl EnumData {
    pub fn from_ast(node: ast::EnumDef) -> Option<Self> {
        let name = node.name()?.to_string();
        let variants = node.variants().filter_map(VariantData::from_ast).collect();

        Some(EnumData { name, variants })
    }
}

impl FlagsData {
    pub fn from_ast(node: ast::FlagsDef) -> Option<Self> {
        let name = node.name()?.to_string();
        let flags = node.flags().filter_map(FlagData::from_ast).collect();

        Some(FlagsData { name, flags })
    }
}

impl FieldData {
    pub fn from_ast(node: ast::Field) -> Option<Self> {
        let name = node.name()?.to_string();
        let ty = node.ty().and_then(TypeData::from_ast)?;

        Some(FieldData { name, ty })
    }
}

impl VariantData {
    pub fn from_ast(node: ast::Variant) -> Option<Self> {
        let name = node.name()?.to_string();
        let fields = node.fields().filter_map(FieldData::from_ast).collect();

        Some(VariantData { name, fields })
    }
}

impl FlagData {
    pub fn from_ast(node: ast::Flag) -> Option<Self> {
        let name = node.name()?.to_string();

        Some(FlagData { name })
    }
}

impl TypeData {
    pub fn from_ast(node: ast::Type) -> Option<Self> {
        let name = node.name()?.to_string();
        let generics = node.generics().filter_map(TypeData::from_ast).collect();

        Some(TypeData { name, generics })
    }
}
