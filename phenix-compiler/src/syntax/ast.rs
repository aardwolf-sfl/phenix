#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    StructDef,
    EnumDef,
    FlagsDef,
    Import,
    Alias,
    Field,
    Variant,
    Flag,
    Name,
    Type,

    Ident,
    String,
    KwStruct,
    KwEnum,
    KwFlags,
    KwImport,
    KwFrom,
    KwAs,
    Star,
    CurlyBracketLeft,
    CurlyBracketRight,
    Colon,
    Comma,
    AngleBracketLeft,
    AngleBracketRight,

    WhiteSpace,
    Comment,

    Error,
    Root,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Language {}

impl rowan::Language for Language {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::Root as u16);
        unsafe { core::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Language>;

pub trait HasName: rowan::ast::AstNode<Language = Language> {
    fn name(&self) -> Option<Name>;
}

pub use nodes::*;

mod nodes {
    use core::fmt;

    use rowan::ast::{support, AstNode};

    use super::*;

    macro_rules! def_ast_node {
        ($node:ident) => {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $node(SyntaxNode);

            impl AstNode for $node {
                type Language = Language;

                fn can_cast(kind: SyntaxKind) -> bool
                where
                    Self: Sized,
                {
                    kind == SyntaxKind::$node
                }

                fn cast(node: SyntaxNode) -> Option<Self>
                where
                    Self: Sized,
                {
                    if node.kind() == SyntaxKind::$node {
                        Some(Self(node))
                    } else {
                        None
                    }
                }

                fn syntax(&self) -> &SyntaxNode {
                    &self.0
                }
            }
        };
    }

    def_ast_node!(Root);
    def_ast_node!(StructDef);
    def_ast_node!(EnumDef);
    def_ast_node!(FlagsDef);
    def_ast_node!(Field);
    def_ast_node!(Variant);
    def_ast_node!(Flag);
    def_ast_node!(Name);
    def_ast_node!(Type);
    def_ast_node!(Import);
    def_ast_node!(Alias);

    macro_rules! impl_has_name {
        ($node:ident) => {
            impl HasName for $node {
                fn name(&self) -> Option<Name> {
                    support::child(self.syntax())
                }
            }
        };
    }

    impl_has_name!(StructDef);
    impl_has_name!(EnumDef);
    impl_has_name!(FlagsDef);
    impl_has_name!(Field);
    impl_has_name!(Variant);
    impl_has_name!(Flag);
    impl_has_name!(Type);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ItemDef(SyntaxNode);

    #[derive(Debug)]
    pub enum ItemDefKind {
        Struct(StructDef),
        Enum(EnumDef),
        Flags(FlagsDef),
    }

    impl AstNode for ItemDef {
        type Language = Language;

        fn can_cast(kind: SyntaxKind) -> bool
        where
            Self: Sized,
        {
            StructDef::can_cast(kind) || EnumDef::can_cast(kind) || FlagsDef::can_cast(kind)
        }

        fn cast(node: SyntaxNode) -> Option<Self>
        where
            Self: Sized,
        {
            if Self::can_cast(node.kind()) {
                Some(Self(node))
            } else {
                None
            }
        }

        fn syntax(&self) -> &SyntaxNode {
            &self.0
        }
    }

    impl ItemDef {
        pub fn kind(&self) -> ItemDefKind {
            StructDef::cast(self.0.clone())
                .map(ItemDefKind::Struct)
                .or_else(|| EnumDef::cast(self.0.clone()).map(ItemDefKind::Enum))
                .or_else(|| FlagsDef::cast(self.0.clone()).map(ItemDefKind::Flags))
                .unwrap()
        }
    }

    impl HasName for ItemDef {
        fn name(&self) -> Option<Name> {
            match self.kind() {
                ItemDefKind::Struct(def) => def.name(),
                ItemDefKind::Enum(def) => def.name(),
                ItemDefKind::Flags(def) => def.name(),
            }
        }
    }

    impl Root {
        pub fn defs(&self) -> impl Iterator<Item = ItemDef> + '_ {
            self.syntax().children().filter_map(ItemDef::cast)
        }

        pub fn imports(&self) -> impl Iterator<Item = Import> + '_ {
            self.syntax().children().filter_map(Import::cast)
        }
    }

    impl StructDef {
        pub fn fields(&self) -> impl Iterator<Item = Field> + '_ {
            self.syntax().children().filter_map(Field::cast)
        }
    }

    impl EnumDef {
        pub fn variants(&self) -> impl Iterator<Item = Variant> + '_ {
            self.syntax().children().filter_map(Variant::cast)
        }
    }

    impl FlagsDef {
        pub fn flags(&self) -> impl Iterator<Item = Flag> + '_ {
            self.syntax().children().filter_map(Flag::cast)
        }
    }

    impl Field {
        pub fn ty(&self) -> Option<Type> {
            support::child(self.syntax())
        }
    }

    impl Variant {
        pub fn fields(&self) -> impl Iterator<Item = Field> + '_ {
            self.syntax().children().filter_map(Field::cast)
        }
    }

    impl Type {
        pub fn generics(&self) -> impl Iterator<Item = Type> + '_ {
            self.syntax().children().filter_map(Type::cast)
        }
    }

    impl fmt::Display for Name {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.syntax().text())
        }
    }

    impl Import {
        pub fn aliases(&self) -> impl Iterator<Item = Alias> + '_ {
            self.syntax().children().filter_map(Alias::cast)
        }

        pub fn is_star(&self) -> bool {
            support::token(self.syntax(), SyntaxKind::Star).is_some()
        }

        pub fn path(&self) -> Option<String> {
            support::token(self.syntax(), SyntaxKind::String).and_then(|token| {
                Some(
                    token
                        .text()
                        .strip_prefix('"')?
                        .strip_suffix('"')?
                        .to_string(),
                )
            })
        }
    }

    impl Alias {
        pub fn name_from(&self) -> Option<Name> {
            self.syntax().children().find_map(Name::cast)
        }

        pub fn name_to(&self) -> Option<Name> {
            self.syntax().children().filter_map(Name::cast).last()
        }
    }
}
