use crate::calculator::{Edge, Graph, Parser};
use core::fmt::Debug;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use syn;

fn get_node<T: Hash>(ast: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    ast.hash(&mut hasher);

    hasher.finish()
}

type ParseResult<T> = Result<T, Box<dyn Error + 'static>>;

fn get_ast(file_path: String) -> ParseResult<syn::File> {
    println!("processing path: {}", file_path);

    let mut src: String = String::new();
    let mut file: File = File::open(&file_path)?;
    file.read_to_string(&mut src)?;

    Ok(syn::parse_file(&src)?)
}

pub struct ASTGraphParser;

impl Parser for ASTGraphParser {
    fn parse(&mut self, file: String) -> Graph {
        let ast = get_ast(file).ok().unwrap();
        let ast_graph = ASTGraph::new(ast);
        let edges: Vec<Edge> = ast_graph.edges.into_iter().map(|t| t.into()).collect();
        Graph::new(edges)
    }
}

#[derive(Debug, PartialEq)]
struct ASTGraph {
    nodes: HashSet<u64>,
    edges: Vec<(u64, u64)>,
}

impl ASTGraph {
    fn new(ast: syn::File) -> ASTGraph {
        let mut ast_graph = ASTGraph {
            nodes: HashSet::new(),
            edges: vec![],
        };
        ast_graph.parse_file(ast, 0);

        ast_graph
    }

    fn checked_update(&mut self, node: u64, parent: u64) -> bool {
        self.edges.push((parent, node));

        if self.nodes.contains(&node) {
            false
        } else {
            self.nodes.insert(node);
            true
        }
    }

    fn parse_file(&mut self, ast: syn::File, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            for item in ast.items {
                self.parse_item(item, node);
            }
        }
    }

    fn parse_item(&mut self, ast: syn::Item, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            match ast {
                syn::Item::ExternCrate(ast) => self.parse_item_extern_crate(ast, node),
                syn::Item::Use(ast) => self.parse_item_use(ast, node),
                syn::Item::Static(ast) => self.parse_item_static(ast, node),
                syn::Item::Const(ast) => self.parse_item_const(ast, node),
                syn::Item::Fn(ast) => self.parse_item_fn(ast, node),
                syn::Item::Mod(ast) => self.parse_item_mod(ast, node),
                syn::Item::ForeignMod(ast) => self.parse_item_foreign_mod(ast, node),
                syn::Item::Type(ast) => self.parse_item_type(ast, node),
                syn::Item::Struct(ast) => self.parse_item_struct(ast, node),
                syn::Item::Enum(ast) => self.parse_item_enum(ast, node),
                syn::Item::Union(ast) => self.parse_item_union(ast, node),
                syn::Item::Trait(ast) => self.parse_item_trait(ast, node),
                syn::Item::TraitAlias(ast) => self.parse_item_trait_alias(ast, node),
                syn::Item::Impl(ast) => self.parse_item_impl(ast, node),
                syn::Item::Macro(ast) => self.parse_item_macro(ast, node),
                syn::Item::Macro2(ast) => self.parse_item_macro2(ast, node),
                syn::Item::Verbatim(_) => { /*skipping since i cann't access*/ }
                syn::Item::__TestExhaustive(_) => { /*skipping private things*/ }
            }
        }
    }

    // TODO: add sub checks
    fn parse_item_extern_crate(&mut self, ast: syn::ItemExternCrate, parent: u64) {
        let node = get_node(&ast);
        self.checked_update(node, parent);
    }

    fn parse_item_use(&mut self, ast: syn::ItemUse, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            self.parse_use_tree(ast.tree, node);
        }
    }

    fn parse_item_static(&mut self, ast: syn::ItemStatic, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            self.parse_type(*ast.ty, node);
            self.parse_expr(*ast.expr, node);
        }
    }

    fn parse_item_const(&mut self, ast: syn::ItemConst, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            self.parse_type(*ast.ty, node);
            self.parse_expr(*ast.expr, node);
        }
    }

    fn parse_item_fn(&mut self, ast: syn::ItemFn, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            self.parse_block(*ast.block, node);
        }
    }

    fn parse_item_mod(&mut self, ast: syn::ItemMod, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            if let Some((_, items)) = ast.content {
                for item in items {
                    self.parse_item(item, node);
                }
            }
        }
    }

    fn parse_item_foreign_mod(&mut self, ast: syn::ItemForeignMod, parent: u64) {
        let node = get_node(&ast);
        if self.checked_update(node, parent) {
            for item in ast.items {
                self.parse_foreign_item(item, node);
            }
        }
    }

    fn parse_item_type(&mut self, _: syn::ItemType, _: u64) {}

    fn parse_item_struct(&mut self, _: syn::ItemStruct, _: u64) {}

    fn parse_item_enum(&mut self, _: syn::ItemEnum, _: u64) {}

    fn parse_item_union(&mut self, _: syn::ItemUnion, _: u64) {}

    fn parse_item_trait(&mut self, _: syn::ItemTrait, _: u64) {}

    fn parse_item_trait_alias(&mut self, _: syn::ItemTraitAlias, _: u64) {}

    fn parse_item_impl(&mut self, _: syn::ItemImpl, _: u64) {}

    fn parse_item_macro(&mut self, _: syn::ItemMacro, _: u64) {}

    fn parse_item_macro2(&mut self, _: syn::ItemMacro2, _: u64) {}

    fn parse_use_tree(&mut self, _: syn::UseTree, _: u64) {}

    fn parse_type(&mut self, _: syn::Type, _: u64) {}

    fn parse_expr(&mut self, _: syn::Expr, _: u64) {}

    fn parse_block(&mut self, _: syn::Block, _: u64) {}

    fn parse_foreign_item(&mut self, _: syn::ForeignItem, _: u64) {}
}
