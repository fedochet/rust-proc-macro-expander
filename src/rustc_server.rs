extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::bridge::{server, TokenTree};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::vec::IntoIter;
use std::iter::FromIterator;

use proc_macro::{Delimiter, Spacing, Level, LineColumn};

//#[derive(Clone)]
//pub struct TokenStream;
type TokenStream = proc_macro2::TokenStream;

pub struct TokenStreamBuilder {
    acc: TokenStream
}

impl TokenStreamBuilder {
    fn new() -> TokenStreamBuilder {
        TokenStreamBuilder {
            acc: TokenStream::new()
        }
    }

    fn push(&mut self, stream: TokenStream) {
        self.acc.extend(stream.into_iter())
    }

    fn build(self) -> TokenStream {
        self.acc
    }
}

#[derive(Clone)]
pub struct TokenStreamIter {
    trees: IntoIter<proc_macro2::TokenTree>
}

//#[derive(Clone)]
//pub struct Group;
type Group = proc_macro2::Group;

type MultiSpan = syntax_pos::MultiSpan;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct MyPunct(u32);

#[derive(Clone)]
struct MyPunctData(proc_macro2::Punct);

impl Hash for MyPunctData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hash_span(&self.0.span(), hasher);
        self.0.as_char().hash(hasher);
    }
}

impl Eq for MyPunctData {}

impl PartialEq for MyPunctData {
    fn eq(&self, other: &Self) -> bool {
        let punct = &self.0;
        let other = &other.0;
        return punct.span().eq(&other.span())
            && punct.as_char() == other.as_char()
            && punct.spacing() == other.spacing();
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct MyIdent(u32);

#[derive(Clone)]
struct MyIdentData(proc_macro2::Ident);

fn hash_span<H: Hasher>(span: &proc_macro2::Span, hasher: &mut H) {
    let start = span.start();
    start.line.hash(hasher);
    start.column.hash(hasher);

    let end = span.end();
    end.line.hash(hasher);
    end.column.hash(hasher);
}

impl Hash for MyIdentData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher)
    }
}

impl Eq for MyIdentData {}

impl PartialEq for MyIdentData {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

//#[derive(Clone)]
//pub struct Literal;
type Literal = proc_macro2::Literal;

//#[derive(Clone)]
//pub struct SourceFile;
type SourceFile = proc_macro2::SourceFile;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct MySpan(u32);

#[derive(Copy, Clone)]
struct MySpanData(proc_macro2::Span);

impl Hash for MySpanData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let column = self.0.start();
        column.line.hash(hasher);
        column.column.hash(hasher);
    }
}

impl Eq for MySpanData {}

impl PartialEq for MySpanData {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[derive(Default)]
struct MySpanInterner {
    spans: HashMap<MySpanData, u32>,
    span_data: Vec<MySpanData>,
}

impl MySpanInterner {
    fn intern(&mut self, data: &MySpanData) -> u32 {
        if let Some(index) = self.spans.get(data) {
            return *index;
        }

        let index = self.spans.len() as u32;
        self.span_data.push(*data);
        self.spans.insert(*data, index);

        index
    }

    fn get(&self, index: u32) -> &MySpanData {
        &self.span_data[index as usize]
    }
}

#[derive(Default)]
struct MyIdentInterner {
    idents: HashMap<MyIdentData, u32>,
    ident_data: Vec<MyIdentData>,
}

impl MyIdentInterner {
    fn intern(&mut self, data: &MyIdentData) -> u32 {
        if let Some(index) = self.idents.get(data) {
            return *index;
        }

        let index = self.idents.len() as u32;
        self.ident_data.push(data.clone());
        self.idents.insert(data.clone(), index);

        index
    }

    fn get(&self, index: u32) -> &MyIdentData {
        &self.ident_data[index as usize]
    }

    fn get_mut(&mut self, index: u32) -> &mut MyIdentData {
        self.ident_data.get_mut(index as usize).expect("Should be consistent")
    }
}

#[derive(Default)]
struct MyPunctInterner {
    puncts: HashMap<MyPunctData, u32>,
    punct_data: Vec<MyPunctData>,
}

impl MyPunctInterner {
    fn intern(&mut self, data: &MyPunctData) -> u32 {
        if let Some(index) = self.puncts.get(data) {
            return *index;
        }

        let index = self.puncts.len() as u32;
        self.punct_data.push(data.clone());
        self.puncts.insert(data.clone(), index);

        index
    }

    fn get(&self, index: u32) -> &MyPunctData {
        &self.punct_data[index as usize]
    }

    fn get_mut(&mut self, index: u32) -> &mut MyPunctData {
        self.punct_data.get_mut(index as usize).expect("Should be consistent")
    }
}


#[derive(Default)]
pub struct Rustc {
    span_interner: MySpanInterner,
    ident_interner: MyIdentInterner,
    punct_interner: MyPunctInterner,
//    def_side: MySpan,
//    call_site: MySpan,
}


impl server::Types for Rustc {
    type TokenStream = TokenStream;
    type TokenStreamBuilder = TokenStreamBuilder;
    type TokenStreamIter = TokenStreamIter;
    type Group = Group;
    type Punct = MyPunct;
    type Ident = MyIdent;
    type Literal = Literal;
    type SourceFile = SourceFile;
    type Diagnostic = proc_macro::Diagnostic;
    type Span = MySpan;
    type MultiSpan = MultiSpan;
}

impl server::TokenStream for Rustc {
    fn new(&mut self) -> Self::TokenStream {
        Self::TokenStream::new()
    }

    fn is_empty(&mut self, stream: &Self::TokenStream) -> bool {
        stream.is_empty()
    }
    fn from_str(&mut self, src: &str) -> Self::TokenStream {
        Self::TokenStream::from_str(src).expect("cannot parse string")
    }
    fn to_string(&mut self, stream: &Self::TokenStream) -> String {
        stream.to_string()
    }
    fn from_token_tree(
        &mut self,
        tree: TokenTree<Self::Group, Self::Punct, Self::Ident, Self::Literal>,
    ) -> Self::TokenStream {
        match tree {
            TokenTree::Group(group) => {
                let tree = proc_macro2::TokenTree::from(group);
                Self::TokenStream::from_iter(vec![tree])
            }

            TokenTree::Ident(MyIdent(index)) => {
                let MyIdentData(ident) = self.ident_interner.get(index).clone();
                let tree = proc_macro2::TokenTree::from(ident);
                Self::TokenStream::from_iter(vec![tree])
            }

            TokenTree::Literal(group) => {
                let tree = proc_macro2::TokenTree::from(group);
                Self::TokenStream::from_iter(vec![tree])
            }

            TokenTree::Punct(MyPunct(index)) => {
                let MyPunctData(punct) = self.punct_interner.get(index).clone();
                let tree = proc_macro2::TokenTree::from(punct);
                Self::TokenStream::from_iter(vec![tree])
            }
        }
    }

    fn into_iter(&mut self, stream: Self::TokenStream) -> Self::TokenStreamIter {
        let trees: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();
        TokenStreamIter { trees: trees.into_iter() }
    }
}

impl server::TokenStreamBuilder for Rustc {
    fn new(&mut self) -> Self::TokenStreamBuilder {
        Self::TokenStreamBuilder::new()
    }
    fn push(&mut self, builder: &mut Self::TokenStreamBuilder, stream: Self::TokenStream) {
        builder.push(stream)
    }
    fn build(&mut self, builder: Self::TokenStreamBuilder) -> Self::TokenStream {
        builder.build()
    }
}

impl server::TokenStreamIter for Rustc {
    fn next(
        &mut self,
        iter: &mut Self::TokenStreamIter,
    ) -> Option<TokenTree<Self::Group, Self::Punct, Self::Ident, Self::Literal>> {
        iter.trees.next().map(|tree| {
            match tree {
                proc_macro2::TokenTree::Group(group) => {
                    TokenTree::Group(group)
                }

                proc_macro2::TokenTree::Ident(ident) => {
                    TokenTree::Ident(MyIdent(self.ident_interner.intern(&MyIdentData(ident))))
                }

                proc_macro2::TokenTree::Literal(literal) => {
                    TokenTree::Literal(literal)
                }

                proc_macro2::TokenTree::Punct(punct) => {
                    TokenTree::Punct(MyPunct(self.punct_interner.intern(&MyPunctData(punct))))
                }
            }
        })
    }
}

fn delim_to_internal(d: proc_macro::Delimiter) -> proc_macro2::Delimiter {
    match d {
        proc_macro::Delimiter::Parenthesis => proc_macro2::Delimiter::Parenthesis,
        proc_macro::Delimiter::Brace => proc_macro2::Delimiter::Brace,
        proc_macro::Delimiter::Bracket => proc_macro2::Delimiter::Bracket,
        proc_macro::Delimiter::None => proc_macro2::Delimiter::None,
    }
}

fn delim_to_external(d: proc_macro2::Delimiter) -> proc_macro::Delimiter {
    match d {
        proc_macro2::Delimiter::Parenthesis => proc_macro::Delimiter::Parenthesis,
        proc_macro2::Delimiter::Brace => proc_macro::Delimiter::Brace,
        proc_macro2::Delimiter::Bracket => proc_macro::Delimiter::Bracket,
        proc_macro2::Delimiter::None => proc_macro::Delimiter::None,
    }
}

fn spacing_to_internal(spacing: proc_macro::Spacing) -> proc_macro2::Spacing {
    match spacing {
        proc_macro::Spacing::Alone => { proc_macro2::Spacing::Alone }
        proc_macro::Spacing::Joint => { proc_macro2::Spacing::Joint }
    }
}

fn spacing_to_external(spacing: proc_macro2::Spacing) -> proc_macro::Spacing {
    match spacing {
        proc_macro2::Spacing::Alone => { proc_macro::Spacing::Alone }
        proc_macro2::Spacing::Joint => { proc_macro::Spacing::Joint }
    }
}

impl server::Group for Rustc {
    fn new(&mut self, delimiter: Delimiter, stream: Self::TokenStream) -> Self::Group {
        Self::Group::new(delim_to_internal(delimiter), stream)
    }
    fn delimiter(&mut self, group: &Self::Group) -> Delimiter {
        delim_to_external(group.delimiter())
    }
    fn stream(&mut self, group: &Self::Group) -> Self::TokenStream {
        group.stream()
    }
    fn span(&mut self, group: &Self::Group) -> Self::Span {
        MySpan(self.span_interner.intern(&MySpanData(group.span())))
    }

    fn set_span(&mut self, group: &mut Self::Group, span: Self::Span) {
        let MySpanData(span) = *self.span_interner.get(span.0);
        group.set_span(span);
    }

    fn span_open(&mut self, group: &Self::Group) -> Self::Span {
        MySpan(self.span_interner.intern(&MySpanData(group.span_open())))
    }

    fn span_close(&mut self, group: &Self::Group) -> Self::Span {
        MySpan(self.span_interner.intern(&MySpanData(group.span_close())))
    }
}

impl server::Punct for Rustc {
    fn new(&mut self, ch: char, spacing: Spacing) -> Self::Punct {
        MyPunct(self.punct_interner.intern(&MyPunctData(proc_macro2::Punct::new(ch, spacing_to_internal(spacing)))))
    }

    fn as_char(&mut self, punct: Self::Punct) -> char {
        let MyPunctData(punct) = self.punct_interner.get(punct.0);
        punct.as_char()
    }
    fn spacing(&mut self, punct: Self::Punct) -> Spacing {
        let MyPunctData(punct) = self.punct_interner.get(punct.0);
        spacing_to_external(punct.spacing())
    }
    fn span(&mut self, punct: Self::Punct) -> Self::Span {
        let MyPunctData(punct) = self.punct_interner.get(punct.0);
        MySpan(self.span_interner.intern(&MySpanData(punct.span())))
    }
    fn with_span(&mut self, punct: Self::Punct, span: Self::Span) -> Self::Punct {
        let MySpanData(span) = *self.span_interner.get(span.0);
        self.punct_interner.get_mut(punct.0).0.set_span(span);
        punct
    }
}

impl server::Ident for Rustc {
    fn new(&mut self, string: &str, span: Self::Span, _is_raw: bool) -> Self::Ident {
        let MySpanData(span) = self.span_interner.get(span.0);
        MyIdent(self.ident_interner.intern(&MyIdentData(proc_macro2::Ident::new(string, *span))))
    }

    fn span(&mut self, ident: Self::Ident) -> Self::Span {
        let MyIdentData(ident) = self.ident_interner.get(ident.0);
        MySpan(self.span_interner.intern(&MySpanData(ident.span())))
    }
    fn with_span(&mut self, ident: Self::Ident, span: Self::Span) -> Self::Ident {
        let MySpanData(span) = *self.span_interner.get(span.0);
        self.ident_interner.get_mut(ident.0).0.set_span(span);
        ident
    }
}

impl server::Literal for Rustc {
    // FIXME(eddyb) `Literal` should not expose internal `Debug` impls.
    fn debug(&mut self, _literal: &Self::Literal) -> String {
        unimplemented!("Literal::debug")
    }
    fn integer(&mut self, _n: &str) -> Self::Literal {
        unimplemented!("integer")
    }
    fn typed_integer(&mut self, _n: &str, _kind: &str) -> Self::Literal {
        unimplemented!("typed_integer")
    }
    fn float(&mut self, _n: &str) -> Self::Literal {
        unimplemented!("flat")
    }
    fn f32(&mut self, _n: &str) -> Self::Literal {
        unimplemented!("f32")
    }
    fn f64(&mut self, _n: &str) -> Self::Literal {
        unimplemented!("f64")
    }
    fn string(&mut self, string: &str) -> Self::Literal {
        Self::Literal::string(string)
    }
    fn character(&mut self, ch: char) -> Self::Literal {
        Self::Literal::character(ch)
    }
    fn byte_string(&mut self, bytes: &[u8]) -> Self::Literal {
        Self::Literal::byte_string(bytes)
    }

    fn span(&mut self, literal: &Self::Literal) -> Self::Span {
        MySpan(self.span_interner.intern(&MySpanData(literal.span())))
    }

    fn set_span(&mut self, literal: &mut Self::Literal, span: Self::Span) {
        let MySpanData(span) = *self.span_interner.get(span.0);
        literal.set_span(span);
    }
}

impl server::SourceFile for Rustc {
    fn eq(&mut self, file1: &Self::SourceFile, file2: &Self::SourceFile) -> bool {
        file1.eq(file2)
    }
    fn path(&mut self, file: &Self::SourceFile) -> String {
//        match file.path() {
//            FileName::Real(ref path) => path
//                .to_str()
//                .expect("non-UTF8 file path in `proc_macro::SourceFile::path`")
//                .to_string(),
        /*_ =>*/
//        }
        String::from(file.path().to_str().expect("non-UTF8 file path in `proc_macro::SourceFile::path`"))
    }
    fn is_real(&mut self, file: &Self::SourceFile) -> bool {
        file.is_real()
    }
}

impl server::Diagnostic for Rustc {
    fn new(&mut self, _level: Level, _msg: &str, _: Self::MultiSpan) -> Self::Diagnostic {
        unimplemented!("new")
    }
//    fn new_span(&mut self, level: Level, msg: &str, span: Self::Span) -> Self::Diagnostic {
////        let MySpanData(span) = *self.span_interner.get(span.0);
////
////        Self::Diagnostic::spanned(span, level, msg)
//        unimplemented!("new_span")
//    }

    fn sub(&mut self, _diag: &mut Self::Diagnostic, _level: Level, _msg: &str, _: Self::MultiSpan) {
        unimplemented!("sub")
    }

//    fn sub_span(&mut self, diag: &mut Self::Diagnostic, level: Level, msg: &str, span: Self::Span) {
//        unimplemented!("sub_span")
//    }

    fn emit(&mut self, diag: Self::Diagnostic) {
        diag.emit()
    }
}

impl server::Span for Rustc {
    fn debug(&mut self, span: Self::Span) -> String {
        unimplemented!("Span::debug")
    }
    fn def_site(&mut self) -> Self::Span {
        MySpan(self.span_interner.intern(&MySpanData(proc_macro2::Span::def_site())))
    }
    fn call_site(&mut self) -> Self::Span {
        MySpan(self.span_interner.intern(&MySpanData(proc_macro2::Span::call_site())))
    }
    fn source_file(&mut self, span: Self::Span) -> Self::SourceFile {
        let MySpanData(span) = self.span_interner.get(span.0);
        span.source_file()
    }
    fn parent(&mut self, span: Self::Span) -> Option<Self::Span> {
//        let MySpanData(span) = *self.span_interner.get(span.0);
//        if let Some(span) = span.parent() {
//            return Some(MySpan(self.span_interner.intern(&MySpanData(span))))
//        }

        None
    }
    fn source(&mut self, span: Self::Span) -> Self::Span {
//        let MySpanData(span) = *self.span_interner.get(span.0);
//
//        MySpan(self.span_interner.intern(&MySpanData(span.source())))
        span
    }
    fn start(&mut self, span: Self::Span) -> LineColumn {
        let MySpanData(span) = *self.span_interner.get(span.0);

//        span.start()
        span.unstable().start()
    }
    fn end(&mut self, span: Self::Span) -> LineColumn {
        let MySpanData(span) = *self.span_interner.get(span.0);

        span.unstable().end()
    }
    fn join(&mut self, first: Self::Span, second: Self::Span) -> Option<Self::Span> {
        let MySpanData(first) = *self.span_interner.get(first.0);
        let MySpanData(second) = *self.span_interner.get(second.0);

        if let Some(join) = first.join(second) {
            return Some(MySpan(self.span_interner.intern(&MySpanData(join))));
        }

        None
    }
    fn resolved_at(&mut self, span: Self::Span, at: Self::Span) -> Self::Span {
        let MySpanData(span) = *self.span_interner.get(span.0);
        let MySpanData(at) = *self.span_interner.get(at.0);
        let resolved_at = span.resolved_at(at);

        MySpan(self.span_interner.intern(&MySpanData(resolved_at)))
    }
}

impl server::MultiSpan for Rustc {
    fn new(&mut self) -> Self::MultiSpan {
        unimplemented!("MultiSpan::new is not implemented");
    }

    fn push(&mut self, _other: &mut Self::MultiSpan, _span: Self::Span) {
        unimplemented!("MultiSpan::new is not implemented");
    }
}

//impl server::Span for Rustc {
//    fn debug(&mut self, _span: Self::Span) -> String {
//        unimplemented!("Span::debug")
//    }
//    fn def_site(&mut self) -> Self::Span {
//        unimplemented!("def_site")
//    }
//    fn call_site(&mut self) -> Self::Span {
//        MySpan(self.span_interner.intern(&MySpanData(proc_macro2::Span::call_site())))
//    }
//    fn source_file(&mut self, _span: Self::Span) -> Self::SourceFile {
//        unimplemented!("source_file")
//    }
//    fn parent(&mut self, _span: Self::Span) -> Option<Self::Span> {
//        unimplemented!("parent")
//    }
//    fn source(&mut self, _span: Self::Span) -> Self::Span {
//        unimplemented!("source")
//    }
//    fn start(&mut self, _span: Self::Span) -> LineColumn {
//        unimplemented!("start")
//    }
//    fn end(&mut self, _span: Self::Span) -> LineColumn {
//        unimplemented!("end")
//    }
//    fn join(&mut self, _first: Self::Span, _second: Self::Span) -> Option<Self::Span> {
//        unimplemented!("join")
//    }
//    fn resolved_at(&mut self, _span: Self::Span, _at: Self::Span) -> Self::Span {
//        unimplemented!("Span::resolved_at")
//    }
//}
