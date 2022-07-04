use logos::{Lexer, Logos};
use rowan::{ast::AstNode, GreenNode, GreenNodeBuilder};

use super::{
    ast::{self, SyntaxKind},
    lexer::Token,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parse {
    root: GreenNode,
    errors: Vec<String>,
}

pub fn parse(source: &str) -> Parse {
    Parser::new(source).parse()
}

impl Parse {
    pub fn root(&self) -> ast::Root {
        ast::Root::cast(ast::SyntaxNode::new_root(self.root.clone())).unwrap()
    }

    pub fn errors(&self) -> &[String] {
        self.errors.as_slice()
    }
}

#[derive(Debug)]
struct Parser<'source> {
    lexer: Lexer<'source, Token<'source>>,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<String>,
    peeked: Option<Token<'source>>,
}

impl<'source> Parser<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            peeked: None,
        }
    }

    pub fn parse(mut self) -> Parse {
        self.parse_node(SyntaxKind::Root, |p| {
            p.eat_trivia();

            while let Some(token) = p.peek() {
                match token {
                    Token::KwStruct => p.parse_struct().recover(RecoverContext::StructDef, p),
                    Token::KwEnum => p.parse_enum().recover(RecoverContext::EnumDef, p),
                    Token::KwFlags => p.parse_flags().recover(RecoverContext::FlagsDef, p),
                    Token::KwImport => p.parse_import().recover(RecoverContext::Import, p),
                    Token::KwFrom
                    | Token::KwAs
                    | Token::Star
                    | Token::CurlyBracketLeft
                    | Token::CurlyBracketRight
                    | Token::Colon
                    | Token::Comma
                    | Token::AngleBracketLeft
                    | Token::AngleBracketRight
                    | Token::Ident(_)
                    | Token::String(_) => {
                        p.raise_unexpected();
                        p.make_error();
                        Some(())
                    }
                    Token::Error => {
                        p.errors.push(format!("lexer error @ {:?}", p.span()));
                        p.make_error();
                        Some(())
                    }
                    Token::WhiteSpace | Token::Newline | Token::Comment => unreachable!(),
                };

                p.eat_trivia();
            }

            Some(())
        });

        Parse {
            root: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn span(&self) -> (usize, usize) {
        self.lexer.extras.line_col(self.lexer.span().start)
    }

    fn peek(&mut self) -> Option<Token<'source>> {
        if self.peeked.is_none() {
            self.peeked = self.lexer.next();
        }

        self.peeked
    }

    fn bump(&mut self) {
        match self.peeked {
            Some(token) => {
                self.builder.token(token.into(), self.lexer.slice());
                self.peeked = None;
            }
            None => {
                if let Some(token) = self.lexer.next() {
                    self.builder.token(token.into(), self.lexer.slice());
                }
            }
        }
    }

    fn bump_until<F>(&mut self, until: F)
    where
        F: Fn(Token<'source>) -> bool,
    {
        while let Some(peeked) = self.peek() {
            if until(peeked) {
                break;
            } else {
                self.bump();
            }
        }
    }

    fn expect<F>(&mut self, predicate: F) -> Option<Token<'source>>
    where
        F: FnOnce(Token<'source>) -> bool,
    {
        self.peek().and_then(|token| {
            if predicate(token) {
                self.bump();
                Some(token)
            } else {
                None
            }
        })
    }

    fn recover(&mut self, ctx: RecoverContext) {
        self.raise_unexpected();
        self.make_error();

        match ctx {
            RecoverContext::StructDef => self.bump_until(|token| token.is_item_def()),
            RecoverContext::EnumDef => self.bump_until(|token| token.is_item_def()),
            RecoverContext::FlagsDef => self.bump_until(|token| token.is_item_def()),
            RecoverContext::Import => self.bump_until(|token| token.is_item_def()),
        }
    }

    fn parse_node<F, R>(&mut self, kind: SyntaxKind, parser: F) -> Option<R>
    where
        F: FnOnce(&mut Parser<'source>) -> Option<R>,
    {
        self.builder.start_node(kind.into());
        let ret = parser(self);
        self.builder.finish_node();
        ret
    }

    fn make_error(&mut self) {
        self.builder.start_node(SyntaxKind::Error.into());
        self.bump();
        self.builder.finish_node();
    }

    fn eat_trivia(&mut self) {
        while matches!(
            self.peek(),
            Some(Token::WhiteSpace | Token::Newline | Token::Comment)
        ) {
            self.bump();
        }
    }

    fn raise_unexpected(&mut self) {
        match self.peek() {
            Some(token) => {
                self.errors
                    .push(format!("unexpected token: {:?} @ {:?}", token, self.span()))
            }
            None => self.errors.push("unexpected eof".to_string()),
        }
    }

    fn parse_struct(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::StructDef, |p| {
            p.expect(|token| token == Token::KwStruct)?;
            p.eat_trivia();

            p.parse_name()?;
            p.eat_trivia();

            p.expect(|token| token == Token::CurlyBracketLeft)?;
            p.eat_trivia();

            p.parse_def_body(Self::parse_field)?;

            Some(())
        })
    }

    fn parse_enum(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::EnumDef, |p| {
            p.expect(|token| token == Token::KwEnum)?;
            p.eat_trivia();

            p.parse_name()?;
            p.eat_trivia();

            p.expect(|token| token == Token::CurlyBracketLeft)?;
            p.eat_trivia();

            p.parse_def_body(Self::parse_variant)?;

            Some(())
        })
    }

    fn parse_flags(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::FlagsDef, |p| {
            p.expect(|token| token == Token::KwFlags)?;
            p.eat_trivia();

            p.parse_name()?;
            p.eat_trivia();

            p.expect(|token| token == Token::CurlyBracketLeft)?;
            p.eat_trivia();

            p.parse_def_body(Self::parse_flag)?;

            Some(())
        })
    }

    fn parse_import(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::Import, |p| {
            p.expect(|token| token == Token::KwImport)?;
            p.eat_trivia();

            let import_kind = p.peek().and_then(|token| match token {
                Token::Ident(_) => Some(ImportKind::Names),
                Token::Star => Some(ImportKind::All),
                _ => None,
            })?;

            match import_kind {
                ImportKind::Names => loop {
                    p.parse_node(SyntaxKind::Alias, |p| {
                        p.parse_name()?;
                        p.eat_trivia();

                        if p.expect(|token| token == Token::KwAs).is_some() {
                            p.eat_trivia();

                            p.parse_name()?;
                            p.eat_trivia();
                        }

                        Some(())
                    })?;

                    match p.peek() {
                        Some(Token::Comma) => {
                            p.bump();
                            p.eat_trivia();
                        }
                        Some(Token::KwFrom) => break,
                        _ => return None,
                    }
                },
                ImportKind::All => {
                    p.expect(|token| token == Token::Star)?;
                    p.eat_trivia();
                }
            }

            p.expect(|token| token == Token::KwFrom)?;
            p.eat_trivia();

            p.expect(|token| matches!(token, Token::String(_)))?;
            Some(())
        })
    }

    fn parse_name(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::Name, |p| {
            p.expect(|token| matches!(token, Token::Ident(_)))?;
            Some(())
        })
    }

    fn parse_def_body<F>(&mut self, item_parser: F) -> Option<()>
    where
        F: Fn(&mut Self) -> Option<()>,
    {
        loop {
            match self.peek()? {
                Token::Ident(_) => {
                    item_parser(self)?;
                    self.eat_trivia();

                    if self.expect(|token| token == Token::Comma).is_none() {
                        self.eat_trivia();
                        self.expect(|token| token == Token::CurlyBracketRight)?;
                        break;
                    }

                    self.eat_trivia();
                }
                Token::CurlyBracketRight => {
                    self.bump();
                    break;
                }
                _ => return None,
            }
        }

        Some(())
    }

    fn parse_variant(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::Variant, |p| {
            p.parse_name()?;
            p.eat_trivia();

            match p.peek()? {
                Token::CurlyBracketLeft => {
                    p.bump();
                    p.eat_trivia();

                    p.parse_def_body(Self::parse_field)?;
                }
                Token::Comma => {}
                _ => return None,
            }

            Some(())
        })
    }

    fn parse_field(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::Field, |p| {
            p.parse_name()?;
            p.eat_trivia();

            p.expect(|token| token == Token::Colon)?;
            p.eat_trivia();

            p.parse_type()?;

            Some(())
        })
    }

    fn parse_flag(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::Flag, |p| {
            p.parse_name()?;
            Some(())
        })
    }

    fn parse_type(&mut self) -> Option<()> {
        self.parse_node(SyntaxKind::Type, |p| {
            p.parse_name()?;
            p.eat_trivia();

            if p.expect(|token| token == Token::AngleBracketLeft).is_some() {
                loop {
                    p.eat_trivia();

                    p.parse_type()?;
                    p.eat_trivia();

                    match p
                        .expect(|token| matches!(token, Token::Comma | Token::AngleBracketRight))?
                    {
                        Token::Comma => {}
                        Token::AngleBracketRight => break,
                        _ => unreachable!(),
                    }
                }
            }

            Some(())
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RecoverContext {
    StructDef,
    EnumDef,
    FlagsDef,
    Import,
}

trait RecoverExt {
    fn recover(self, ctx: RecoverContext, parser: &mut Parser<'_>) -> Self;
}

impl<T> RecoverExt for Option<T> {
    fn recover(self, ctx: RecoverContext, parser: &mut Parser<'_>) -> Self {
        if self.is_none() {
            parser.recover(ctx);
        }

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ImportKind {
    Names,
    All,
}

impl Token<'_> {
    fn is_item_def(&self) -> bool {
        matches!(self, Token::KwStruct | Token::KwEnum | Token::KwFlags)
    }
}

impl<'source> From<Token<'source>> for SyntaxKind {
    fn from(token: Token<'source>) -> Self {
        match token {
            Token::KwStruct => SyntaxKind::KwStruct,
            Token::KwEnum => SyntaxKind::KwEnum,
            Token::KwFlags => SyntaxKind::KwFlags,
            Token::KwImport => SyntaxKind::KwImport,
            Token::KwFrom => SyntaxKind::KwFrom,
            Token::KwAs => SyntaxKind::KwAs,
            Token::Star => SyntaxKind::Star,
            Token::CurlyBracketLeft => SyntaxKind::CurlyBracketLeft,
            Token::CurlyBracketRight => SyntaxKind::CurlyBracketRight,
            Token::Colon => SyntaxKind::Colon,
            Token::Comma => SyntaxKind::Comma,
            Token::AngleBracketLeft => SyntaxKind::AngleBracketLeft,
            Token::AngleBracketRight => SyntaxKind::AngleBracketRight,
            Token::Ident(_) => SyntaxKind::Ident,
            Token::String(_) => SyntaxKind::String,
            Token::WhiteSpace | Token::Newline => SyntaxKind::WhiteSpace,
            Token::Comment => SyntaxKind::Comment,
            Token::Error => SyntaxKind::Error,
        }
    }
}

impl<'source> From<Token<'source>> for rowan::SyntaxKind {
    fn from(token: Token<'source>) -> Self {
        let kind: SyntaxKind = token.into();
        kind.into()
    }
}
