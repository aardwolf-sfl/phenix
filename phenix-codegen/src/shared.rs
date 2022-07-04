use indexmap::IndexMap;
use rustc_hash::FxHashMap;

use crate::{Module, ModuleId, Project, UserType, UserTypeId};

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
