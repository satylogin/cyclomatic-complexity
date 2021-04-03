use std::error::Error;
use std::fs::File;
use std::io::Read;
use syn;

pub fn calculate_complexity(file_path: String) -> i64 {
    let ast = ASTGraph::new(file_path);

    ast.edges - ast.nodes + 2 * ast.connected_components
}

#[derive(Debug)]
struct ASTGraph {
    pub connected_components: i64,
    pub nodes: i64,
    pub edges: i64,
}

impl ASTGraph {
    pub fn new(file_path: String) -> ASTGraph {
        let mut ast_graph = ASTGraph {
            connected_components: 0,
            nodes: 0,
            edges: 0,
        };
        let ast = get_ast(file_path).ok().unwrap();
        println!("{:#?}", ast);
        ast_graph.parse(ast);

        ast_graph
    }

    fn parse(&mut self, ast: syn::File) {
        for item in ast.items {
            match item {
                _ => println!("Do not know how to process yet"),
            }
        }
    }
}

type ParseResult<T> = Result<T, Box<dyn Error + 'static>>;

fn get_ast(file_path: String) -> ParseResult<syn::File> {
    println!("processing path: {}", file_path);

    let mut src: String = String::new();
    let mut file: File = File::open(&file_path)?;
    file.read_to_string(&mut src)?;

    Ok(syn::parse_file(&src)?)
}
