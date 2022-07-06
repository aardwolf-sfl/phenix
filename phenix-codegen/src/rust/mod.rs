use convert_case::{Case, Casing};

use crate::{
    shared::{self, ModuleTree, ProjectContext},
    Attribute, BuiltinType, EnumType, Field, FlagsType, Module, ModuleId, StructType, Type, TypeId,
    UserType, Variant,
};

pub fn generate(ctx: ProjectContext) -> String {
    Generator::new(ctx).generate()
}

struct Generator {
    ctx: ProjectContext,
}

impl Generator {
    fn new(ctx: ProjectContext) -> Self {
        Self { ctx }
    }

    fn generate(self) -> String {
        let module_tree = ModuleTree::new(self.ctx.project());
        let mut content = String::new();

        self.generate_module(&mut content, &module_tree);

        // parse + unparse roundtrip is not only useful for nicely-formatted
        // output without too much effort on our side, but also validates the we
        // generated actually valid Rust code (at least syntactically).
        let parsed = syn::parse_file(&content).expect("valid Rust code was generated");
        prettyplease::unparse(&parsed)
    }

    fn generate_module(&self, content: &mut String, module: &ModuleTree<'_>) {
        if !module.is_root() {
            content.push_str("pub mod ");
            content.push_str(module.name());
            content.push_str(" {");
        }

        if !module.is_empty() {
            content.push_str("use phenix_runtime::prelude::*;");

            let module_id = module.id().expect("module with types must have id");

            for ty in module.types() {
                self.generate_user_type(content, module_id, ty);
            }
        }

        for child in module.children() {
            self.generate_module(content, child);
        }

        if !module.is_root() {
            content.push('}');
        }
    }

    fn generate_user_type(&self, content: &mut String, module: ModuleId, ty: &UserType) {
        match ty {
            UserType::Struct(ty) => self.generate_struct_type(content, module, ty),
            UserType::Enum(ty) => self.generate_enum_type(content, module, ty),
            UserType::Flags(ty) => self.generate_flags_type(content, ty),
        }
    }

    fn generate_attributes(&self, content: &mut String, attrs: &[Attribute]) {
        for attr in attrs.iter() {
            match attr {
                Attribute::NonExhaustive => content.push_str("#[non_exhaustive]"),
            }
        }
    }

    fn generate_struct_type(&self, content: &mut String, module: ModuleId, ty: &StructType) {
        content.push_str("#[derive(Encodable, Decodable, Debug, Clone, PartialEq)]");

        self.generate_attributes(content, &ty.attrs);

        content.push_str("pub struct ");
        content.push_str(&ty.name.to_case(Case::Pascal));
        content.push_str(" {");

        for field in ty.fields.iter() {
            self.generate_field(content, module, field, true);
        }

        content.push('}');
    }

    fn generate_enum_type(&self, content: &mut String, module: ModuleId, ty: &EnumType) {
        content.push_str("#[derive(Encodable, Decodable, Debug, Clone, PartialEq)]");

        self.generate_attributes(content, &ty.attrs);

        content.push_str("pub enum ");
        content.push_str(&ty.name.to_case(Case::Pascal));
        content.push_str(" {");

        for variant in ty.variants.iter() {
            self.generate_variant(content, module, variant);
        }

        content.push('}');
    }

    fn generate_flags_type(&self, content: &mut String, ty: &FlagsType) {
        content.push_str("#[derive(IsFlag, Debug, Clone, Copy, PartialEq)]");

        self.generate_attributes(content, &ty.attrs);

        let pascal_name = ty.name.to_case(Case::Pascal);

        content.push_str("pub enum ");
        content.push_str(&pascal_name);
        content.push_str("Flag");
        content.push_str(" {");

        for flag in ty.flags.iter() {
            content.push_str(&flag.to_case(Case::Pascal));
            content.push(',');
        }

        content.push('}');

        content.push_str("#[allow(dead_code)]");
        content.push_str("pub type ");
        content.push_str(&pascal_name);
        content.push_str(" = ::phenix_runtime::Flags<");
        content.push_str(&pascal_name);
        content.push_str("Flag");
        content.push_str(">;");
    }

    fn generate_field(&self, content: &mut String, module: ModuleId, field: &Field, vis: bool) {
        if vis {
            content.push_str("pub ");
        }

        content.push_str(&field.name.to_case(Case::Snake));
        content.push(':');
        self.generate_type(content, module, &field.ty);
        content.push(',');
    }

    fn generate_variant(&self, content: &mut String, module: ModuleId, variant: &Variant) {
        content.push_str(&variant.name.to_case(Case::Pascal));

        if !variant.fields.is_empty() {
            content.push_str(" {");

            for field in variant.fields.iter() {
                self.generate_field(content, module, field, false);
            }

            content.push('}');
        }

        content.push(',');
    }

    fn generate_type(&self, content: &mut String, module: ModuleId, ty: &Type) {
        match ty.id {
            TypeId::Builtin(BuiltinType::Bool) => content.push_str("bool"),
            TypeId::Builtin(BuiltinType::U8) => content.push_str("u8"),
            TypeId::Builtin(BuiltinType::U16) => content.push_str("u16"),
            TypeId::Builtin(BuiltinType::U32) => content.push_str("u32"),
            TypeId::Builtin(BuiltinType::U64) => content.push_str("u64"),
            TypeId::Builtin(BuiltinType::I8) => content.push_str("i8"),
            TypeId::Builtin(BuiltinType::I16) => content.push_str("i16"),
            TypeId::Builtin(BuiltinType::I32) => content.push_str("i32"),
            TypeId::Builtin(BuiltinType::I64) => content.push_str("i64"),
            TypeId::Builtin(BuiltinType::F32) => content.push_str("f32"),
            TypeId::Builtin(BuiltinType::F64) => content.push_str("f64"),
            TypeId::Builtin(BuiltinType::Uint) => content.push_str("::phenix_runtime::Uint"),
            TypeId::Builtin(BuiltinType::Sint) => content.push_str("::phenix_runtime::Sint"),
            TypeId::Builtin(BuiltinType::Float) => content.push_str("::phenix_runtime::Float"),
            TypeId::Builtin(BuiltinType::String) => content.push_str("::std::string::String"),
            TypeId::Builtin(BuiltinType::Vector) => content.push_str("::std::vec::Vec"),
            TypeId::Builtin(BuiltinType::Stream) => content.push_str("::phenix_runtime::Stream"),
            TypeId::User(id) => {
                let current_module = self.ctx.module(module).unwrap();
                let source_module = self.ctx.find_module(id).unwrap();
                let user_ty = self.ctx.find_type(id).unwrap();

                if source_module != current_module {
                    self.generate_relative_path(content, current_module, source_module);
                    content.push_str("::");
                }

                content.push_str(user_ty.name());
            }
        }

        if !ty.generics.is_empty() {
            content.push('<');

            shared::punctuated(
                ty.generics.iter(),
                content,
                |content, ty| self.generate_type(content, module, ty),
                |content| content.push(','),
            );

            content.push('>');
        }
    }

    fn generate_relative_path(&self, content: &mut String, from: &Module, to: &Module) {
        let from = from.path.as_slice();
        let to = to.path.as_slice();

        content.push_str("self");

        for _ in 0..from.len() {
            content.push_str("::super");
        }

        for component in to.iter() {
            content.push_str("::");
            content.push_str(component);
        }
    }
}
