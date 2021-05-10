use crate::parsers::error::{ParseError, ParseErrorKind};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use syn;

type ParseResult<T> = Result<T, Box<dyn Error + 'static>>;

fn get_ast(file_path: String) -> ParseResult<syn::File> {
    let mut src: String = String::new();
    let mut file: File = File::open(&file_path)?;
    file.read_to_string(&mut src)?;

    Ok(syn::parse_file(&src)?)
}

#[derive(Debug)]
pub enum ComplexityNodeKind {
    Fn,
    Method,
    Impl,
    File,
}

impl fmt::Display for ComplexityNodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct ComplexityNode {
    pub name: String,
    pub kind: ComplexityNodeKind,
    pub complexity: usize,
    pub children: Vec<ComplexityNode>,
}

impl ComplexityNode {
    fn new(name: String, kind: ComplexityNodeKind) -> ComplexityNode {
        ComplexityNode {
            name,
            kind,
            complexity: 0,
            children: vec![],
        }
    }

    fn with_complexity(mut self, complexity: usize) -> ComplexityNode {
        self.complexity = complexity;
        self
    }

    fn add_child(&mut self, child: ComplexityNode) {
        self.children.push(child);
    }
}

#[derive(Debug)]
pub struct ComplexityTree {
    pub root: ComplexityNode,
}

impl ComplexityTree {
    pub fn generate(file_path: String) -> ParseResult<ComplexityTree> {
        let file: syn::File = get_ast(file_path.clone())?;

        let mut root = ComplexityNode::new(file_path, ComplexityNodeKind::File);
        process_file(file, &mut root);

        Ok(ComplexityTree { root })
    }
}

/// parse ast to get complexity from valid blocks
// TODO: add macros complexity later i.e. Macro, Macro2
fn process_file(ast: syn::File, parent: &mut ComplexityNode) {
    for item in ast.items {
        match item {
            syn::Item::Fn(ast) => process_item_fn(ast, parent),
            syn::Item::Impl(ast) => process_item_impl(ast, parent),
            syn::Item::Mod(_) => {}
            syn::Item::Trait(_) => {}
            _ => {}
        }
    }
}

fn process_item_fn(ast: syn::ItemFn, parent: &mut ComplexityNode) {
    let node = ComplexityNode::new(ast.sig.ident.to_string(), ComplexityNodeKind::Fn)
        .with_complexity((*ast.block).process());

    parent.add_child(node);
}

fn process_item_impl(ast: syn::ItemImpl, parent: &mut ComplexityNode) {
    let mut node = ComplexityNode::new(
        get_impl_resolved_name(&ast).ok().unwrap().to_string(),
        ComplexityNodeKind::Impl,
    );

    for item in ast.items {
        match item {
            syn::ImplItem::Method(ast) => process_impl_item_method(ast, &mut node),
            _ => {}
        }
    }

    parent.add_child(node);
}

fn process_impl_item_method(ast: syn::ImplItemMethod, parent: &mut ComplexityNode) {
    let node = ComplexityNode::new(ast.sig.ident.to_string(), ComplexityNodeKind::Method)
        .with_complexity(ast.block.process());

    parent.add_child(node);
}

fn get_impl_resolved_name(ast: &syn::ItemImpl) -> ParseResult<syn::Ident> {
    match &*ast.self_ty {
        syn::Type::Path(type_path) => Ok(type_path.path.segments[0].ident.clone()),
        _ => Err(Box::new(
            ParseError::kind(ParseErrorKind::NoMatches)
                .msg(String::from("Identifier not found for impl")),
        )),
    }
}

trait Process {
    fn process(self) -> usize;
}

impl Process for syn::Block {
    fn process(self) -> usize {
        let mut complexity: usize = 0;
        for stmt in self.stmts {
            match stmt {
                // syn::Stmt::Local(local) => println!("{:#?}", local),
                // syn::Stmt::Item(item) => println!("{:#?}", item),
                syn::Stmt::Expr(inner) => complexity += inner.process(),
                // syn::Stmt::Semi(expr, semi) => println!("{:#?}, {:#?}", expr, semi),
                _ => {}
            };
        }

        complexity
    }
}

impl Process for syn::Expr {
    fn process(self) -> usize {
        let mut complexity: usize = 0;
        match self {
            syn::Expr::Array(inner) => complexity += inner.process(),
            syn::Expr::Assign(inner) => complexity += inner.process(),
            syn::Expr::AssignOp(inner) => complexity += inner.process(),
            syn::Expr::Block(inner) => complexity += inner.process(),
            syn::Expr::Break(inner) => complexity += inner.process(),
            syn::Expr::If(inner) => complexity += inner.process(),
            _ => {}
        }

        complexity
    }
}

impl Process for syn::ExprArray {
    fn process(self) -> usize {
        let mut complexity: usize = 0;

        for elem in self.elems {
            complexity += elem.process();
        }

        complexity
    }
}

impl Process for syn::ExprAssign {
    fn process(self) -> usize {
        let mut complexity: usize = 0;

        complexity += (*(self.left)).process();
        complexity += (*(self.right)).process();

        complexity
    }
}

impl Process for syn::ExprAssignOp {
    fn process(self) -> usize {
        let mut complexity: usize = 0;

        complexity += (*(self.left)).process();
        complexity += (*(self.right)).process();

        complexity
    }
}

impl Process for syn::ExprBlock {
    fn process(self) -> usize {
        self.block.process()
    }
}

impl Process for syn::ExprBreak {
    fn process(self) -> usize {
        let mut complexity: usize = 1;

        if let Some(expr) = self.expr {
            complexity += (*expr).process();
        }

        complexity
    }
}

impl Process for syn::ExprIf {
    fn process(self) -> usize {
        let mut complexity: usize = 1;

        complexity += self.then_branch.process();

        if let Some((_, expr)) = self.else_branch {
            complexity += (*expr).process();
        }

        complexity
    }
}
