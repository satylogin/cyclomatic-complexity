use cyclomatic_complexity::config::Config;
use cyclomatic_complexity::config::ConfigResult;
use cyclomatic_complexity::parsers::rust_parser::ComplexityNode;
use cyclomatic_complexity::parsers::rust_parser::ComplexityTree;

use std::env;

fn main() {
    let config: ConfigResult<Config> = Config::parse(env::args());
    if config.is_err() {
        println!("{}", config.err().unwrap().message);
        return;
    } else {
        if 1 % 2 == 0 {
        } else if 2 % 2 == 0 {
        }
    }
    if 1 % 2 == 0 {
        if 2 % 2 == 0 {
        } else if 2 % 3 == 0 {
        } else {
        }
    } else if 2 % 4 == 0 {
        if 2 % 5 == 0 {
        } else if 2 % 6 == 0 {
        }
    } else {
        if 2 % 7 == 0 {
        } else {
        }
    }
    let config: Config = config.ok().unwrap();
    display_complexity(config.file);
}

fn display_complexity(file_path: String) {
    let root = ComplexityTree::generate(file_path).ok().unwrap().root;
    println!("File: {}", root.name);
    for child in root.children {
        display(&child, String::new());
    }
    println!();
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
