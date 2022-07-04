use logos::Logos;

#[derive(Debug, Default)]
pub struct LexerExtras {
    line_breaks: Vec<usize>,
}

impl LexerExtras {
    pub fn line_col(&self, byte_pos: usize) -> (usize, usize) {
        // Handle first line.
        if self.line_breaks.is_empty() || self.line_breaks[0] > byte_pos {
            return (1, byte_pos + 1);
        }

        // Handle the rest of lines.
        let line_idx = match self.line_breaks.binary_search(&byte_pos) {
            Ok(idx) => idx,
            Err(idx) => idx - 1,
        };

        let line = line_idx + 2;
        let col = byte_pos - self.line_breaks[line_idx] + 1;

        (line, col)
    }
}

#[derive(Debug, Logos, Clone, Copy, PartialEq, Eq, Hash)]
#[logos(extras = LexerExtras)]
pub enum Token<'source> {
    #[token("struct")]
    KwStruct,

    #[token("enum")]
    KwEnum,

    #[token("flags")]
    KwFlags,

    #[token("import")]
    KwImport,

    #[token("from")]
    KwFrom,

    #[token("as")]
    KwAs,

    #[token("*")]
    Star,

    #[token("{")]
    CurlyBracketLeft,

    #[token("}")]
    CurlyBracketRight,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("<")]
    AngleBracketLeft,

    #[token(">")]
    AngleBracketRight,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Ident(&'source str),

    #[regex("\"[^\"]*\"", |lex| lex.slice())]
    String(&'source str),

    #[regex("//[^\n\r]*")]
    Comment,

    #[regex(r"[ \t\f]+")]
    WhiteSpace,

    #[regex(r"\n|\r\n", |lex| lex.extras.line_breaks.push(lex.span().end))]
    Newline,

    #[error]
    Error,
}
