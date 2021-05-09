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

pub fn display_complexity(file_path: String) {
    let tree = ComplexityTree::generate(file_path).ok().unwrap().root;
    display(&tree, String::new());
}

fn display(node: &ComplexityNode, path: String) {
    let mut path_here: String = path;
    if !path_here.is_empty() {
        path_here += " > ";
    }
    path_here += node.kind.to_string().as_str();
    path_here += ": ";
    path_here += node.name.as_str();

    if node.children.is_empty() {
        println!("[{}] Complexity => {}", path_here, node.complexity);
    } else {
        for child in node.children.iter() {
            display(child, path_here.clone());
        }
    }
}

#[derive(Debug)]
enum ComplexityNodeKind {
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
struct ComplexityNode {
    name: String,
    kind: ComplexityNodeKind,
    complexity: usize,
    children: Vec<ComplexityNode>,
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
struct ComplexityTree {
    root: ComplexityNode,
}

impl ComplexityTree {
    fn generate(file_path: String) -> ParseResult<ComplexityTree> {
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
        .with_complexity(process_block(*ast.block));

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
        .with_complexity(process_block(ast.block));

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

fn process_block(ast: syn::Block) -> usize {
    let mut complexity: usize = 0;
    for stmt in ast.stmts {
        match stmt {
            // syn::Stmt::Local(local) => println!("{:#?}", local),
            // syn::Stmt::Item(item) => println!("{:#?}", item),
            syn::Stmt::Expr(expr) => complexity += process_expr(expr),
            // syn::Stmt::Semi(expr, semi) => println!("{:#?}, {:#?}", expr, semi),
            _ => {}
        };
    }

    complexity
}

fn process_expr(expr: syn::Expr) -> usize {
    let mut complexity: usize = 0;
    match expr {
        syn::Expr::If(expr_if) => {
            complexity += process_expr_if(expr_if);
        }
        syn::Expr::Block(expr_block) => {
            complexity += process_expr_block(expr_block);
        }
        _ => {}
    }

    complexity
}

fn process_expr_if(expr_if: syn::ExprIf) -> usize {
    let mut complexity: usize = 1;

    complexity += process_block(expr_if.then_branch);

    if let Some((_, expr)) = expr_if.else_branch {
        complexity += process_expr(*expr);
    }

    complexity
}

fn process_expr_block(expr_block: syn::ExprBlock) -> usize {
    process_block(expr_block.block)
}
