use std::error::Error;
use std::fs::File;
use std::io::Read;
use syn;

pub fn calculate_complexity(file_path: String) -> i64 {
    let ast = get_ast(file_path).ok().unwrap();
    let ast = ASTGraph::new(ast);
    println!("{:?}", ast);
    ast.edges - ast.nodes + 2 * ast.connected_components
}

#[derive(Debug, PartialEq)]
pub struct ASTGraph {
    pub connected_components: i64,
    pub nodes: i64,
    pub edges: i64,
}

impl ASTGraph {
    pub fn new(ast: syn::File) -> ASTGraph {
        let mut ast_graph = ASTGraph {
            connected_components: 0,
            nodes: 0,
            edges: 0,
        };
        ast_graph.parse(ast);
        ast_graph
    }

    fn parse(&mut self, ast: syn::File) {
        for item in ast.items {
            self.connected_components += 1;
            self.nodes += 1;
            dbg!(&item);
            match item {
                syn::Item::Use(item) => self.parse_item_use(item),
                _ => println!("Do not know how to parse yet"),
            }
        }
    }

    fn parse_item_use(&mut self, ast: syn::ItemUse) {
        self.edges += 1;
        self.nodes += 1;
        match ast.tree {
            syn::UseTree::Name(ast) => self.parse_use_name(ast),
            syn::UseTree::Path(ast) => self.parse_use_path(ast),
            _ => println!("Do not know how parse_item_use to parse yet"),
        }
    }

    fn parse_use_path(&mut self, ast: syn::UsePath) {
        self.edges += 1;
        self.nodes += 1;
        match *ast.tree {
            syn::UseTree::Name(ast) => self.parse_use_name(ast),
            syn::UseTree::Path(ast) => self.parse_use_path(ast),
            _ => println!("dont know how to parse_use_path yet"),
        }
    }

    fn parse_use_name(&mut self, _ast: syn::UseName) {}
}

type ParseResult<T> = Result<T, Box<dyn Error + 'static>>;

fn get_ast(file_path: String) -> ParseResult<syn::File> {
    println!("processing path: {}", file_path);

    let mut src: String = String::new();
    let mut file: File = File::open(&file_path)?;
    file.read_to_string(&mut src)?;

    Ok(syn::parse_file(&src)?)
}

#[cfg(test)]
mod tests {
    use crate::cyclomatic::ASTGraph;
    use rstest::rstest;

    // expected has connected_components, edges, nodes
    #[rstest(input, expected,
        case("use a;", (1, 1, 2)),
        case("use a::b;", (1, 2, 3)),
        case("use a::b::c;", (1, 3, 4)),
        case("use a::b::c::d;", (1, 4, 5)),
        case("use a::b::c::d::e;", (1, 5, 6)),
        case("use a;\nuse a::b;", (2, 3, 5)),
        case("use a::b::c::d;\nuse a;\nuse a::b;", (3, 7, 10)),
    )]
    fn test_liner_use(input: &str, expected: (i64, i64, i64)) {
        let ast = syn::parse_str(input).ok().unwrap();
        let ast = ASTGraph::new(ast);
        assert_eq!(
            ast,
            ASTGraph {
                connected_components: expected.0,
                edges: expected.1,
                nodes: expected.2,
            }
        );
    }
}
