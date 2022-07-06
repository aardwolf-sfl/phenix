use derive_more::Deref;
use indexmap::IndexMap;
use rustc_hash::FxHashMap;

use crate::{Module, ModuleId, Project, Type, TypeId, UserType, UserTypeId};

pub struct ProjectContext {
    project: Project,
    type_to_mod: FxHashMap<UserTypeId, ModuleId>,
}

impl ProjectContext {
    pub fn new(project: Project) -> Self {
        let mut type_to_mod = FxHashMap::default();

        if project
            .modules
            .iter()
            .enumerate()
            .any(|(i, module)| *module.id != i)
        {
            panic!("Module ids is consecutive sequence of numbers starting at 0");
        }

        for module in project.modules.iter() {
            for ty in module.types.iter() {
                if type_to_mod.insert(ty.id(), module.id).is_some() {
                    panic!("Type id {} is not unique", *ty.id());
                }
            }
        }

        // XXX: We now assume perfect validity of the project data. To be more
        // robust against errors, we should perform proper validation of
        // well-formedness.

        Self {
            project,
            type_to_mod,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn module(&self, id: ModuleId) -> Option<&Module> {
        self.project.modules.get(*id)
    }

    pub fn find_module(&self, id: UserTypeId) -> Option<&Module> {
        self.type_to_mod
            .get(&id)
            .map(|mod_id| &self.project.modules[**mod_id])
    }

    pub fn find_type(&self, id: UserTypeId) -> Option<&UserType> {
        self.find_module(id)
            .and_then(|module| module.types.iter().find(|ty| ty.id() == id))
    }
}

impl UserType {
    pub fn id(&self) -> UserTypeId {
        match self {
            UserType::Struct(ty) => ty.id,
            UserType::Enum(ty) => ty.id,
            UserType::Flags(ty) => ty.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            UserType::Struct(ty) => ty.name.as_str(),
            UserType::Enum(ty) => ty.name.as_str(),
            UserType::Flags(ty) => ty.name.as_str(),
        }
    }

    pub fn used_types(&self) -> impl Iterator<Item = &Type> + '_ {
        let iter: Box<dyn Iterator<Item = &Type>> = match self {
            UserType::Struct(ty) => Box::new(ty.fields.iter().map(|field| &field.ty)),
            UserType::Enum(ty) => Box::new(
                ty.variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter().map(|field| &field.ty)),
            ),
            UserType::Flags(_) => Box::new(std::iter::empty()),
        };

        iter
    }
}

pub struct ModuleTree<'a> {
    id: Option<ModuleId>,
    name: String,
    types: &'a [UserType],
    children: IndexMap<String, ModuleTree<'a>>,
}

impl<'a> ModuleTree<'a> {
    pub fn new(project: &'a Project) -> Self {
        let mut root = ModuleTree {
            id: None,
            name: String::new(),
            types: &[],
            children: IndexMap::default(),
        };

        for module in project.modules.iter() {
            let mut node = &mut root;

            for name in module.path.iter().cloned() {
                node = node
                    .children
                    .entry(name.clone())
                    .or_insert_with(|| ModuleTree {
                        id: None,
                        name,
                        types: &[],
                        children: IndexMap::default(),
                    });
            }

            node.id = Some(module.id);
            node.types = module.types.as_slice();
        }

        root
    }

    pub fn id(&self) -> Option<ModuleId> {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn types(&self) -> &[UserType] {
        self.types
    }

    pub fn children(&self) -> impl Iterator<Item = &ModuleTree<'a>> {
        self.children.values()
    }

    pub fn is_root(&self) -> bool {
        self.name.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

#[derive(Debug, Clone, Deref)]
pub struct TypeDependencyOrder(Vec<TypeDependency>);

impl TypeDependencyOrder {
    pub fn new(ctx: &ProjectContext) -> Self {
        let mut marks = FxHashMap::default();
        let mut tree = Vec::new();

        for module in ctx.project().modules.iter() {
            for ty in module.types.iter() {
                type_dependency_order(ty, ctx, &mut marks, &mut tree);
            }
        }

        Self(tree)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeDependency {
    Type(UserTypeId),
    Cycle(UserTypeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mark {
    Temporary,
    Permanent,
}

fn type_dependency_order(
    ty: &UserType,
    ctx: &ProjectContext,
    marks: &mut FxHashMap<UserTypeId, Mark>,
    output: &mut Vec<TypeDependency>,
) {
    match marks.get(&ty.id()) {
        Some(Mark::Temporary) => {
            output.push(TypeDependency::Cycle(ty.id()));
            return;
        }
        Some(Mark::Permanent) => return,
        None => {}
    }

    marks.insert(ty.id(), Mark::Temporary);

    for used_ty in ty.used_types() {
        if let TypeId::User(id) = used_ty.id {
            let used_ty = ctx.find_type(id).unwrap();
            type_dependency_order(used_ty, ctx, marks, output);
        }

        for gen_ty in used_ty.generics.iter() {
            if let TypeId::User(id) = gen_ty.id {
                let gen_ty = ctx.find_type(id).unwrap();
                type_dependency_order(gen_ty, ctx, marks, output);
            }
        }
    }

    marks.insert(ty.id(), Mark::Permanent);
    output.push(TypeDependency::Type(ty.id()));
}

pub fn punctuated<T, I, F, G>(mut iter: I, content: &mut String, mut item_cb: F, mut punct_cb: G)
where
    I: Iterator<Item = T>,
    F: FnMut(&mut String, T),
    G: FnMut(&mut String),
{
    if let Some(item) = iter.next() {
        item_cb(content, item);
    }

    for item in iter {
        punct_cb(content);
        item_cb(content, item);
    }
}

pub fn byte_size(n_bits: usize) -> usize {
    let div = n_bits / u8::BITS as usize;
    let rem = n_bits % u8::BITS as usize;

    if rem > 0 {
        div + 1
    } else {
        div
    }
}
