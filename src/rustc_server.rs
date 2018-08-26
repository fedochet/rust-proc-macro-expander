use proc_macro::bridge::{server, TokenTree};

extern crate proc_macro2;

use syntax::ast;

use syntax_pos::symbol::{Symbol, keywords};
use syntax_pos::{Span, DUMMY_SP};

use self::proc_macro2::{TokenStream};
use syntax::tokenstream::{TokenStreamBuilder, Cursor};
use proc_macro::{Delimiter, Spacing, Level, LineColumn};

use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    sym: Symbol,
    span: Span,
    is_raw: bool,
}

#[derive(Clone)]
pub struct Group {
    delimiter: Delimiter,
    stream: TokenStream,
    span: Span,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Punct {
    ch: char,
    // NB. not using `Spacing` here because it doesn't implement `Hash`.
    joint: bool,
    span: Span,
}

// FIXME(eddyb) `Literal` should not expose internal `Debug` impls.
#[derive(Clone, Debug)]
pub struct Literal {
    suffix: Option<Symbol>,
    span: Span,
}

pub(crate) struct Rustc {}

#[derive(Clone)]
pub struct TokenStreamIter {
    cursor: Cursor,
    stack: Vec<TokenTree<Group, Punct, Ident, Literal>>,
}

#[derive(Clone)]
pub struct SourceFile {}

pub struct Diagnostic {}

impl server::Types for Rustc {
    type TokenStream = TokenStream;
    type TokenStreamBuilder = TokenStreamBuilder;
    type TokenStreamIter = TokenStreamIter;
    type Group = Group;
    type Punct = Punct;
    type Ident = Ident;
    type Literal = Literal;
    type SourceFile = SourceFile;
    type Diagnostic = Diagnostic;
    type Span = Span;
}

impl server::TokenStream for Rustc {
    fn new(&mut self) -> Self::TokenStream {
//        println!("New TokenStream");
        proc_macro2::TokenStream::new()
    }
    fn is_empty(&mut self, stream: &Self::TokenStream) -> bool {
//        println!("IsEmpty");
        stream.is_empty()
    }
    fn from_str(&mut self, src: &str) -> Self::TokenStream {
//        println!("From Str");
        TokenStream::from_str(src).unwrap()
    }
    fn to_string(&mut self, stream: &Self::TokenStream) -> String {
//        println!("To string");
        stream.to_string()
    }
    fn from_token_tree(
        &mut self,
        tree: TokenTree<Self::Group, Self::Punct, Self::Ident, Self::Literal>,
    ) -> Self::TokenStream {
//        println!("From token tree");
        unimplemented!();
    }
    fn into_iter(&mut self, stream: Self::TokenStream) -> Self::TokenStreamIter {
//        println!("Into iter");
        unimplemented!();
    }
}

impl server::TokenStreamBuilder for Rustc {
    fn new(&mut self) -> Self::TokenStreamBuilder {
        TokenStreamBuilder::new()
    }
    fn push(&mut self, builder: &mut Self::TokenStreamBuilder, stream: Self::TokenStream) {
        unimplemented!()
    }
    fn build(&mut self, builder: Self::TokenStreamBuilder) -> Self::TokenStream {
        unimplemented!();
    }
}

impl server::TokenStreamIter for Rustc {
    fn next(
        &mut self,
        iter: &mut Self::TokenStreamIter,
    ) -> Option<TokenTree<Self::Group, Self::Punct, Self::Ident, Self::Literal>> {
        unimplemented!()
    }
}

impl server::Group for Rustc {
    fn new(&mut self, delimiter: Delimiter, stream: Self::TokenStream) -> Self::Group {
        Group {
            delimiter,
            stream,
            span: server::Span::call_site(self),
        }
    }
    fn delimiter(&mut self, group: &Self::Group) -> Delimiter {
        group.delimiter
    }
    fn stream(&mut self, group: &Self::Group) -> Self::TokenStream {
        group.stream.clone()
    }
    fn span(&mut self, group: &Self::Group) -> Self::Span {
        group.span
    }
    fn set_span(&mut self, group: &mut Self::Group, span: Self::Span) {
        group.span = span;
    }
}


impl server::Punct for Rustc {
    fn new(&mut self, ch: char, spacing: Spacing) -> Self::Punct {
        Punct {
            ch,
            joint: spacing == Spacing::Joint,
            span: server::Span::call_site(self),
        }
    }
    fn as_char(&mut self, punct: Self::Punct) -> char {
        punct.ch
    }
    fn spacing(&mut self, punct: Self::Punct) -> Spacing {
        if punct.joint {
            Spacing::Joint
        } else {
            Spacing::Alone
        }
    }
    fn span(&mut self, punct: Self::Punct) -> Self::Span {
        punct.span
    }
    fn with_span(&mut self, punct: Self::Punct, span: Self::Span) -> Self::Punct {
        Punct { span, ..punct }
    }
}

impl server::Ident for Rustc {
    fn new(&mut self, string: &str, span: Self::Span, is_raw: bool) -> Self::Ident {
        let sym = Symbol::intern(string);
        if is_raw
            && (sym == keywords::Underscore.name()
            || ast::Ident::with_empty_ctxt(sym).is_path_segment_keyword())
            {
                panic!("`{:?}` is not a valid raw identifier", string)
            }
        Ident { sym, span, is_raw }
    }
    fn span(&mut self, ident: Self::Ident) -> Self::Span {
        ident.span
    }
    fn with_span(&mut self, ident: Self::Ident, span: Self::Span) -> Self::Ident {
        Ident { span, ..ident }
    }
}

impl server::Literal for Rustc {
    // FIXME(eddyb) `Literal` should not expose internal `Debug` impls.
    fn debug(&mut self, literal: &Self::Literal) -> String {
        format!("{:?}", literal)
    }
    fn integer(&mut self, n: &str) -> Self::Literal {
        unimplemented!()
    }
    fn typed_integer(&mut self, n: &str, kind: &str) -> Self::Literal {
        unimplemented!()
    }
    fn float(&mut self, n: &str) -> Self::Literal {
        unimplemented!()
    }
    fn f32(&mut self, n: &str) -> Self::Literal {
        unimplemented!()
    }
    fn f64(&mut self, n: &str) -> Self::Literal {
        unimplemented!()
    }
    fn string(&mut self, string: &str) -> Self::Literal {
        let mut escaped = String::new();
        for ch in string.chars() {
            escaped.extend(ch.escape_debug());
        }
        unimplemented!()
    }
    fn character(&mut self, ch: char) -> Self::Literal {
        let mut escaped = String::new();
        escaped.extend(ch.escape_unicode());
        unimplemented!()
    }
    fn byte_string(&mut self, bytes: &[u8]) -> Self::Literal {
        unimplemented!()
    }
    fn span(&mut self, literal: &Self::Literal) -> Self::Span {
        literal.span
    }
    fn set_span(&mut self, literal: &mut Self::Literal, span: Self::Span) {
        literal.span = span;
    }
}

impl server::SourceFile for Rustc {
    fn eq(&mut self, file1: &Self::SourceFile, file2: &Self::SourceFile) -> bool {
        unimplemented!()
    }
    fn path(&mut self, file: &Self::SourceFile) -> String {
        unimplemented!()    }
    fn is_real(&mut self, file: &Self::SourceFile) -> bool {
        unimplemented!()
    }
}

impl server::Diagnostic for Rustc {
    fn new(&mut self, level: Level, msg: &str) -> Self::Diagnostic {
        unimplemented!()
    }
    fn new_span(&mut self, level: Level, msg: &str, span: Self::Span) -> Self::Diagnostic {
        unimplemented!()
    }
    fn sub(&mut self, diag: &mut Self::Diagnostic, level: Level, msg: &str) {
        unimplemented!()
    }
    fn sub_span(&mut self, diag: &mut Self::Diagnostic, level: Level, msg: &str, span: Self::Span) {
        unimplemented!()
    }
    fn emit(&mut self, diag: Self::Diagnostic) {
        unimplemented!()
    }
}

impl server::Span for Rustc {
    fn debug(&mut self, span: Self::Span) -> String {
        format!("{:?} bytes({}..{})", span.ctxt(), span.lo().0, span.hi().0)
    }
    fn def_site(&mut self) -> Self::Span {
        unimplemented!()
    }
    fn call_site(&mut self) -> Self::Span {
        unimplemented!()
    }
    fn source_file(&mut self, span: Self::Span) -> Self::SourceFile {
        unimplemented!()
    }
    fn parent(&mut self, span: Self::Span) -> Option<Self::Span> {
        unimplemented!()
    }
    fn source(&mut self, span: Self::Span) -> Self::Span {
        unimplemented!()
    }
    fn start(&mut self, span: Self::Span) -> LineColumn {
        unimplemented!()
    }
    fn end(&mut self, span: Self::Span) -> LineColumn {
        unimplemented!()
    }
    fn join(&mut self, first: Self::Span, second: Self::Span) -> Option<Self::Span> {
        unimplemented!()
    }
    fn resolved_at(&mut self, span: Self::Span, at: Self::Span) -> Self::Span {
        span.with_ctxt(at.ctxt())
    }
}
