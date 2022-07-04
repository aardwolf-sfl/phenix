use std::path::Path;

use phenix_codegen::{
    BuiltinType, EnumType, Field, FlagsType, ModulePath, StructType, Type, TypeId, UserType,
    UserTypeId, Variant,
};

use crate::{
    database::ir::{DefDatabase, ItemId, VfsFileId},
    ir::{ItemData, TypeData},
};

pub fn resolve_module_path(root_dir: &Path, module_file: &Path) -> ModulePath {
    let mut root_iter = root_dir.components().peekable();
    let mut file_iter = module_file.components().peekable();

    // Consume the shared prefix.
    root_iter
        .by_ref()
        .zip(file_iter.by_ref())
        .take_while(|(a, b)| a == b)
        .for_each(|_| {});

    if root_iter.peek().is_some() {
        // The module is higher in the directory hierarchy, so we consider it
        // "external".
        ModulePath(vec!["external".to_string()])
    } else {
        let components = file_iter
            .filter_map(|component| {
                let component = component.as_os_str().to_str()?;
                let component = component.strip_suffix(".phenix").unwrap_or(component);

                let normalized = component
                    .chars()
                    .skip_while(|c| !c.is_ascii_alphabetic())
                    .scan(' ', |state, c| {
                        let c = if c.is_ascii_alphanumeric() {
                            c
                        } else {
                            if *state == '_' {
                                return Some(None);
                            }

                            '_'
                        };

                        *state = c;
                        Some(Some(c))
                    })
                    .flatten()
                    .collect::<String>();

                Some(normalized)
            })
            .collect();

        ModulePath(components)
    }
}

pub fn resolve_type(db: &dyn DefDatabase, module: VfsFileId, ty: &TypeData) -> Option<Type> {
    let id = match ty.name.as_str() {
        "bool" => TypeId::Builtin(BuiltinType::Bool),
        "u8" => TypeId::Builtin(BuiltinType::U8),
        "u16" => TypeId::Builtin(BuiltinType::U16),
        "u32" => TypeId::Builtin(BuiltinType::U32),
        "u64" => TypeId::Builtin(BuiltinType::U64),
        "i8" => TypeId::Builtin(BuiltinType::I8),
        "i16" => TypeId::Builtin(BuiltinType::I16),
        "i32" => TypeId::Builtin(BuiltinType::I32),
        "i64" => TypeId::Builtin(BuiltinType::I64),
        "f32" => TypeId::Builtin(BuiltinType::F32),
        "f64" => TypeId::Builtin(BuiltinType::F64),
        "uint" => TypeId::Builtin(BuiltinType::Uint),
        "sint" => TypeId::Builtin(BuiltinType::Sint),
        "float" => TypeId::Builtin(BuiltinType::Float),
        "string" => TypeId::Builtin(BuiltinType::String),
        "vector" => TypeId::Builtin(BuiltinType::Vector),
        "stream" => TypeId::Builtin(BuiltinType::Stream),
        _ => {
            let item_id = db.module_item_by_name(module, ty.name.clone())?;
            TypeId::User(UserTypeId(item_id.as_usize()))
        }
    };

    let generics = ty
        .generics
        .iter()
        .map(|ty| resolve_type(db, module, ty))
        .collect::<Option<Vec<_>>>()?;

    Some(Type { id, generics })
}

pub fn make_def(db: &dyn DefDatabase, module: VfsFileId, item_id: ItemId) -> Option<UserType> {
    let item = db.lookup_intern_item(item_id).into_inner();
    let id = UserTypeId(item_id.as_usize());

    let def = match item {
        ItemData::Struct(data) => StructType {
            id,
            name: data.name,
            fields: data
                .fields
                .into_iter()
                .map(|field| {
                    resolve_type(db, module, &field.ty).map(|ty| Field {
                        name: field.name,
                        ty,
                        attrs: Vec::new(),
                    })
                })
                .collect::<Option<_>>()?,
            attrs: Vec::new(),
        }
        .into(),
        ItemData::Enum(data) => EnumType {
            id,
            name: data.name,
            variants: data
                .variants
                .into_iter()
                .map(|variant| {
                    variant
                        .fields
                        .into_iter()
                        .map(|field| {
                            resolve_type(db, module, &field.ty).map(|ty| Field {
                                name: field.name,
                                ty,
                                attrs: Vec::new(),
                            })
                        })
                        .collect::<Option<_>>()
                        .map(|fields| Variant {
                            name: variant.name,
                            fields,
                            attrs: Vec::new(),
                        })
                })
                .collect::<Option<_>>()?,
            attrs: Vec::new(),
        }
        .into(),
        ItemData::Flags(data) => FlagsType {
            id,
            name: data.name,
            flags: data.flags.into_iter().map(|flag| flag.name).collect(),
            attrs: Vec::new(),
        }
        .into(),
    };

    Some(def)
}
