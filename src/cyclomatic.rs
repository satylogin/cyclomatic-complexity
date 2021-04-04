use core::fmt::Debug;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use syn;

pub fn calculate_complexity(file_path: String) -> i64 {
    let ast = get_ast(file_path).ok().unwrap();
    let ast = ASTGraph::new(ast);
    dbg!(&ast);
    ast.edges - ast.nodes + 2 * ast.connected_components
}

#[derive(Debug, PartialEq)]
pub struct ASTGraph {
    pub connected_components: i64,
    pub nodes: i64,
    pub edges: i64,
    visited: HashSet<u64>,
}

impl ASTGraph {
    pub fn new(ast: syn::File) -> ASTGraph {
        let mut ast_graph = ASTGraph {
            connected_components: 0,
            nodes: 0,
            edges: 0,
            visited: HashSet::new(),
        };
        ast_graph.parse_file(ast);
        ast_graph
    }

    fn checked_update<T: Hash>(&mut self, ast: T) -> bool {
        self.edges += 1;
        let mut hasher = DefaultHasher::new();
        ast.hash(&mut hasher);
        let addr = hasher.finish();

        if self.visited.contains(&addr) {
            false
        } else {
            self.visited.insert(addr);
            self.nodes += 1;
            true
        }
    }

    fn parse_file(&mut self, ast: syn::File) {
        for item in ast.items {
            self.connected_components += 1;
            self.parse_item(item);
        }
    }

    fn parse_item(&mut self, ast: syn::Item) {
        if self.checked_update(&ast) {
            match ast {
                syn::Item::ExternCrate(ast) => self.parse_item_extern_crate(ast),
                syn::Item::Use(ast) => self.parse_item_use(ast),
                syn::Item::Static(ast) => self.parse_item_static(ast),
                syn::Item::Const(ast) => self.parse_item_const(ast),
                syn::Item::Fn(ast) => self.parse_item_fn(ast),
                syn::Item::Mod(ast) => self.parse_item_mod(ast),
                syn::Item::ForeignMod(ast) => self.parse_item_foreign_mod(ast),
                syn::Item::Type(ast) => self.parse_item_type(ast),
                syn::Item::Struct(ast) => self.parse_item_struct(ast),
                syn::Item::Enum(ast) => self.parse_item_enum(ast),
                syn::Item::Union(ast) => self.parse_item_union(ast),
                syn::Item::Trait(ast) => self.parse_item_trait(ast),
                syn::Item::TraitAlias(ast) => self.parse_item_trait_alias(ast),
                syn::Item::Impl(ast) => self.parse_item_impl(ast),
                syn::Item::Macro(ast) => self.parse_item_macro(ast),
                syn::Item::Macro2(ast) => self.parse_item_macro2(ast),
                syn::Item::Verbatim(_) => { /*skipping since i cann't access*/ }
                syn::Item::__TestExhaustive(_) => { /*skipping private things*/ }
            }
        }
    }

    fn parse_item_extern_crate(&mut self, ast: syn::ItemExternCrate) {
        if self.checked_update(&ast) {
            if ast.rename.is_some() {
                self.nodes += 2;
                self.edges += 2;
            }
        }
    }

    fn parse_item_use(&mut self, ast: syn::ItemUse) {
        if self.checked_update(&ast) {
            self.parse_use_tree(ast.tree);
        }
    }

    fn parse_item_static(&mut self, ast: syn::ItemStatic) {
        if self.checked_update(&ast) {
            self.parse_type(*ast.ty);
            self.parse_expr(*ast.expr);
        }
    }

    fn parse_item_const(&mut self, ast: syn::ItemConst) {
        if self.checked_update(&ast) {
            self.parse_type(*ast.ty);
            self.parse_expr(*ast.expr);
        }
    }

    fn parse_item_fn(&mut self, ast: syn::ItemFn) {
        if self.checked_update(&ast) {
            self.parse_block(*ast.block);
        }
    }

    fn parse_item_mod(&mut self, _: syn::ItemMod) {}

    fn parse_item_foreign_mod(&mut self, _: syn::ItemForeignMod) {}

    fn parse_item_type(&mut self, _: syn::ItemType) {}

    fn parse_item_struct(&mut self, _: syn::ItemStruct) {}

    fn parse_item_enum(&mut self, _: syn::ItemEnum) {}

    fn parse_item_union(&mut self, _: syn::ItemUnion) {}

    fn parse_item_trait(&mut self, _: syn::ItemTrait) {}

    fn parse_item_trait_alias(&mut self, _: syn::ItemTraitAlias) {}

    fn parse_item_impl(&mut self, _: syn::ItemImpl) {}

    fn parse_item_macro(&mut self, _: syn::ItemMacro) {}

    fn parse_item_macro2(&mut self, _: syn::ItemMacro2) {}

    fn parse_use_tree(&mut self, _: syn::UseTree) {}

    fn parse_type(&mut self, _: syn::Type) {}

    fn parse_expr(&mut self, _: syn::Expr) {}

    fn parse_block(&mut self, _: syn::Block) {}
}

type ParseResult<T> = Result<T, Box<dyn Error + 'static>>;

fn get_ast(file_path: String) -> ParseResult<syn::File> {
    println!("processing path: {}", file_path);

    let mut src: String = String::new();
    let mut file: File = File::open(&file_path)?;
    file.read_to_string(&mut src)?;

    Ok(syn::parse_file(&src)?)
}
