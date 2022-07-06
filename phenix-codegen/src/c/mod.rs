use std::env;

use convert_case::{Case, Casing};
use derive_more::From;
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use serde::Serialize;
use tera::{Context as TeraContext, Tera};

use crate::{
    shared::{self, TypeDependencyOrder, TypeDependency, ProjectContext},
    Attribute, BuiltinType, EnumType, Field, FlagsType, Module, StructType, Type, TypeId, UserType,
    Variant,
};

pub fn generate(ctx: ProjectContext) -> String {
    let ctx = &ctx;

    let monos = ctx
        .project()
        .modules
        .iter()
        .flat_map(|module| {
            module.types.iter().flat_map(|ty| {
                ty.used_types().filter_map(|ty| match ty.id {
                    TypeId::Builtin(BuiltinType::Vector) => Some(MonomorphizationContext::Vector(
                        VectorContext::new(&ty.generics[0], ctx),
                    )),
                    TypeId::Builtin(BuiltinType::Stream) => Some(MonomorphizationContext::Stream(
                        StreamContext::new(&ty.generics[0], ctx),
                    )),
                    _ => None,
                })
            })
        })
        .collect::<IndexSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    // Dependency order is important for having fully defined structs as fields
    // of other structs.
    let items = TypeDependencyOrder::new(ctx)
        .iter()
        .copied()
        .map(|dep| {
            let id = match dep {
                TypeDependency::Type(dep) => dep,
                TypeDependency::Cycle(_) => panic!("type cycles are not supported"),
            };

            let module = ctx.find_module(id).unwrap();
            let ty = ctx.find_type(id).unwrap();

            let prefix = module_prefix(module);
            match ty {
                UserType::Struct(ty) => StructContext::new(ty, &prefix, ctx).into(),
                UserType::Enum(ty) => EnumContext::new(ty, &prefix, ctx).into(),
                UserType::Flags(ty) => FlagsContext::new(ty, &prefix).into(),
            }
        })
        .collect::<Vec<_>>();

    let templates = Templates::new();
    let mut content = String::new();

    // Inline the `phenix_runtime.h` header file for easier distribution.
    let header_file = include_str!("../../../phenix-capi/include/phenix_runtime.h");
    content.push_str(header_file);
    content.push_str("\n\n");

    content.push_str(
        r#"
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
"#,
    );

    content.push_str("\n\n");

    for mono in monos.iter() {
        templates.render_monomorphization_decl_to(mono, &mut content);
        content.push('\n');
    }

    for item in items.iter() {
        templates.render_item_decl_to(item, &mut content);
        content.push('\n');
    }

    content.push('\n');

    for mono in monos.iter() {
        templates.render_monomorphization_impl_to(mono, &mut content);
        content.push('\n');
    }

    for item in items.iter() {
        templates.render_item_impl_to(item, &mut content);
        content.push('\n');
    }

    content
}

static PREFIX: Lazy<String> =
    Lazy::new(|| env::var("PHENIX_C_PREFIX").unwrap_or_else(|_| "phenix_generated_".to_string()));

fn module_prefix(module: &Module) -> String {
    let mut prefix = PREFIX.clone();

    if !module.path.is_empty() {
        prefix.push_str(&module.path.join("_"));
        prefix.push('_');
    }

    prefix
}

struct Templates {
    tera: Tera,
}

impl Templates {
    fn new() -> Self {
        let mut tera = Tera::default();

        tera.add_raw_template("vector_impl", include_str!("templates/vector_impl.tera"))
            .expect("valid template");
        tera.add_raw_template("stream_impl", include_str!("templates/stream_impl.tera"))
            .expect("valid template");

        tera.add_raw_template(
            "encode_many_impl",
            include_str!("templates/encode_many_impl.tera"),
        )
        .expect("valid template");

        tera.add_raw_template("struct_decl", include_str!("templates/struct_decl.tera"))
            .expect("valid template");
        tera.add_raw_template("struct_impl", include_str!("templates/struct_impl.tera"))
            .expect("valid template");
        tera.add_raw_template("enum_decl", include_str!("templates/enum_decl.tera"))
            .expect("valid template");
        tera.add_raw_template("enum_impl", include_str!("templates/enum_impl.tera"))
            .expect("valid template");
        tera.add_raw_template("flags_decl", include_str!("templates/flags_decl.tera"))
            .expect("valid template");
        tera.add_raw_template("flags_impl", include_str!("templates/flags_impl.tera"))
            .expect("valid template");

        Self { tera }
    }

    fn render_monomorphization_decl_to(&self, mono: &MonomorphizationContext, output: &mut String) {
        let context = mono.tera_context();

        let rendered = self.tera.render("struct_decl", &context).unwrap();
        output.push_str(&rendered);
    }

    fn render_monomorphization_impl_to(&self, mono: &MonomorphizationContext, output: &mut String) {
        let mut context = mono.tera_context();

        let template = match mono {
            MonomorphizationContext::Vector(_) => "vector_impl",
            MonomorphizationContext::Stream(_) => "stream_impl",
        };

        let rendered = self.tera.render(template, &context).unwrap();
        output.push_str(&rendered);
        output.push('\n');

        if let Some(extension) = EncodeManyContext::from_monomorphization(mono) {
            context.extend(extension.tera_context());

            let rendered = self.tera.render("encode_many_impl", &context).unwrap();
            output.push_str(&rendered);
        }
    }

    fn render_item_decl_to(&self, item: &ItemContext, output: &mut String) {
        let context = item.tera_context();

        let template = match item {
            ItemContext::Struct(_) => "struct_decl",
            ItemContext::Enum(_) => "enum_decl",
            ItemContext::Flags(_) => "flags_decl",
        };

        let rendered = self.tera.render(template, &context).unwrap();
        output.push_str(&rendered);
    }

    fn render_item_impl_to(&self, item: &ItemContext, output: &mut String) {
        let mut context = item.tera_context();

        let template = match item {
            ItemContext::Struct(_) => "struct_impl",
            ItemContext::Enum(_) => "enum_impl",
            ItemContext::Flags(_) => "flags_impl",
        };

        let rendered = self.tera.render(template, &context).unwrap();
        output.push_str(&rendered);
        output.push('\n');

        context.extend(EncodeManyContext::from_item(item).tera_context());

        let rendered = self.tera.render("encode_many_impl", &context).unwrap();
        output.push_str(&rendered);
    }
}

#[derive(Debug, From)]
enum ItemContext {
    Struct(StructContext),
    Enum(EnumContext),
    Flags(FlagsContext),
}

impl ItemContext {
    fn tera_context(&self) -> TeraContext {
        match self {
            ItemContext::Struct(context) => TeraContext::from_serialize(context).unwrap(),
            ItemContext::Enum(context) => TeraContext::from_serialize(context).unwrap(),
            ItemContext::Flags(context) => TeraContext::from_serialize(context).unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
struct StructContext {
    name: String,
    fields: Vec<FieldContext>,
}

impl StructContext {
    fn new(ty: &StructType, prefix: &str, ctx: &ProjectContext) -> Self {
        let mut name = prefix.to_string();
        name.push_str(&ty.name.to_case(Case::Snake));

        let fields = ty
            .fields
            .iter()
            .filter(|field| !field.ty.is_stream())
            .map(|field| FieldContext::new(field, ctx))
            .collect();

        Self { name, fields }
    }
}

#[derive(Debug, Serialize)]
struct EnumContext {
    name: String,
    variants: Vec<VariantContext>,
    has_data: bool,
}

impl EnumContext {
    fn new(ty: &EnumType, prefix: &str, ctx: &ProjectContext) -> Self {
        let mut name = prefix.to_string();
        name.push_str(&ty.name.to_case(Case::Snake));

        let variants = ty
            .variants
            .iter()
            .map(|variant| VariantContext::new(variant, ctx))
            .collect();

        let has_data = ty.variants.iter().any(|variant| !variant.fields.is_empty());

        Self {
            name,
            variants,
            has_data,
        }
    }
}

#[derive(Debug, Serialize)]
struct FlagsContext {
    name: String,
    flags: Vec<String>,
    n_bytes: usize,
    is_exhaustive: bool,
}

impl FlagsContext {
    fn new(ty: &FlagsType, prefix: &str) -> Self {
        let mut name = prefix.to_string();
        name.push_str(&ty.name.to_case(Case::Snake));

        let flags = ty
            .flags
            .iter()
            .map(|flag| flag.to_case(Case::ScreamingSnake))
            .collect();

        let is_exhaustive = !ty
            .attrs
            .iter()
            .any(|attr| attr == &Attribute::NonExhaustive);

        Self {
            name,
            flags,
            // C arrays must have non-zero size. This would happen if the type
            // had zero flags.
            n_bytes: shared::byte_size(ty.flags.len()).max(1),
            is_exhaustive,
        }
    }
}

#[derive(Debug, Serialize)]
struct FieldContext {
    name: String,
    ty: TypeContext,
}

impl FieldContext {
    fn new(field: &Field, ctx: &ProjectContext) -> Self {
        Self {
            name: field.name.to_case(Case::Snake),
            ty: TypeContext::new(&field.ty, ctx),
        }
    }
}

#[derive(Debug, Serialize)]
struct VariantContext {
    name: String,
    fields: Vec<FieldContext>,
    has_data: bool,
}

impl VariantContext {
    fn new(variant: &Variant, ctx: &ProjectContext) -> Self {
        let fields = variant
            .fields
            .iter()
            .filter(|field| !field.ty.is_stream())
            .map(|field| FieldContext::new(field, ctx))
            .collect();

        Self {
            name: variant.name.to_case(Case::Snake),
            fields,
            has_data: !variant.fields.is_empty(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TypeContext {
    c_name: String,
    rt_name: String,
    rt_prefix: String,
    by_ref: bool,
}

impl TypeContext {
    fn new(ty: &Type, ctx: &ProjectContext) -> Self {
        let (mut c_name, mut rt_name, keyword) = match ty.id {
            TypeId::Builtin(builtin)
                if matches!(builtin, BuiltinType::Vector | BuiltinType::Stream) =>
            {
                let (mut c_name, mut rt_name) = builtin.ty_context();

                c_name.insert_str(0, &PREFIX);
                rt_name.insert_str(0, &PREFIX);

                (c_name, rt_name, "struct")
            }
            TypeId::Builtin(builtin) => {
                let (c_name, rt_name) = builtin.ty_context();

                return Self {
                    c_name,
                    rt_name,
                    rt_prefix: "phenix_runtime_".to_string(),
                    by_ref: false,
                };
            }
            TypeId::User(id) => {
                let source_module = ctx.find_module(id).unwrap();
                let user_ty = ctx.find_type(id).unwrap();

                let mut name = module_prefix(source_module);
                name.push_str(&user_ty.name().to_case(Case::Snake));

                (name.clone(), name, user_ty.keyword())
            }
        };

        c_name.insert(0, ' ');
        c_name.insert_str(0, keyword);

        for gen_ty in ty.generics.iter() {
            let gen_ty = TypeContext::new(gen_ty, ctx);

            let c_name_stripped = gen_ty
                .c_name
                .strip_prefix(&*PREFIX)
                .unwrap_or(&gen_ty.c_name);

            c_name.push('_');
            c_name.push_str(c_name_stripped);

            let rt_name_stripped = gen_ty
                .rt_name
                .strip_prefix(&*PREFIX)
                .unwrap_or(&gen_ty.rt_name);

            rt_name.push('_');
            rt_name.push_str(rt_name_stripped);
        }

        c_name.push_str("__");

        Self {
            c_name,
            rt_name,
            rt_prefix: String::new(),
            by_ref: true,
        }
    }
}

impl Type {
    fn is_stream(&self) -> bool {
        matches!(self.id, TypeId::Builtin(BuiltinType::Stream))
    }
}

impl BuiltinType {
    fn ty_context(&self) -> (String, String) {
        let (c_name, rt_name) = match self {
            BuiltinType::Bool => ("bool", "bool"),
            BuiltinType::U8 => ("uint8_t", "u8"),
            BuiltinType::U16 => ("uint16_t", "u16"),
            BuiltinType::U32 => ("uint32_t", "u32"),
            BuiltinType::U64 => ("uint64_t", "u64"),
            BuiltinType::I8 => ("int8_t", "i8"),
            BuiltinType::I16 => ("int16_t", "i16"),
            BuiltinType::I32 => ("int32_t", "i32"),
            BuiltinType::I64 => ("int64_t", "i64"),
            BuiltinType::F32 => ("float", "f32"),
            BuiltinType::F64 => ("double", "f64"),
            BuiltinType::Uint => ("uint64_t", "uint"),
            BuiltinType::Sint => ("int64_t", "sint"),
            BuiltinType::Float => ("double", "float"),
            BuiltinType::String => ("const char *", "string"),
            BuiltinType::Vector => ("vector", "vector"),
            BuiltinType::Stream => ("stream", "stream"),
        };

        (c_name.to_string(), rt_name.to_string())
    }
}

impl UserType {
    fn keyword(&self) -> &'static str {
        match self {
            UserType::Struct(_) => "struct",
            UserType::Enum(ty) => {
                if ty.variants.iter().any(|variant| !variant.fields.is_empty()) {
                    "union"
                } else {
                    "enum"
                }
            }
            UserType::Flags(_) => "struct",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum MonomorphizationContext {
    Vector(VectorContext),
    Stream(StreamContext),
}

impl MonomorphizationContext {
    fn tera_context(&self) -> TeraContext {
        match self {
            MonomorphizationContext::Vector(context) => {
                TeraContext::from_serialize(context).unwrap()
            }
            MonomorphizationContext::Stream(context) => {
                TeraContext::from_serialize(context).unwrap()
            }
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
struct VectorContext {
    name: String,
    c_name: String,
    rt_name: String,
    rt_prefix: String,
}

impl VectorContext {
    fn new(ty: &Type, ctx: &ProjectContext) -> Self {
        let ty = TypeContext::new(ty, ctx);
        let rt_name_stripped = ty.rt_name.strip_prefix(&*PREFIX).unwrap_or(&ty.rt_name);

        Self {
            name: PREFIX.clone() + "vector_" + rt_name_stripped,
            c_name: ty.c_name,
            rt_name: ty.rt_name,
            rt_prefix: ty.rt_prefix,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
struct StreamContext {
    name: String,
    c_name: String,
    rt_name: String,
    rt_prefix: String,
}

impl StreamContext {
    fn new(ty: &Type, ctx: &ProjectContext) -> Self {
        let ty = TypeContext::new(ty, ctx);
        let rt_name_stripped = ty.rt_name.strip_prefix(&*PREFIX).unwrap_or(&ty.rt_name);

        Self {
            name: PREFIX.clone() + "stream_" + rt_name_stripped,
            c_name: ty.c_name,
            rt_name: ty.rt_name,
            rt_prefix: ty.rt_prefix,
        }
    }
}

#[derive(Debug, Serialize)]
struct EncodeManyContext {
    keyword: &'static str,
}

impl EncodeManyContext {
    fn from_item(item: &ItemContext) -> Self {
        let keyword = match item {
            ItemContext::Struct(_) => "struct",
            ItemContext::Enum(item) => {
                if item.has_data {
                    "union"
                } else {
                    "enum"
                }
            }
            ItemContext::Flags(_) => "struct",
        };

        Self { keyword }
    }

    fn from_monomorphization(mono: &MonomorphizationContext) -> Option<Self> {
        let keyword = match mono {
            MonomorphizationContext::Vector(_) => "struct",
            MonomorphizationContext::Stream(_) => return None,
        };

        Some(Self { keyword })
    }

    fn tera_context(&self) -> TeraContext {
        TeraContext::from_serialize(self).unwrap()
    }
}
