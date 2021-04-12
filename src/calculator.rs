use std::collections::HashSet;
use std::convert::From;

pub type Node = u64;

pub struct Edge {
    from: Node,
    to: Node,
}

impl From<(Node, Node)> for Edge {
    fn from(t: (Node, Node)) -> Edge {
        Edge { from: t.0, to: t.1 }
    }
}

pub struct Graph {
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new(edges: Vec<Edge>) -> Graph {
        Graph { edges }
    }

    fn calculate_complexity(&self) -> i32 {
        let edge_count: i32 = self.edges.len() as i32;

        let mut nodes: HashSet<Node> = HashSet::new();

        // we put all nodes in set
        for edge in self.edges.iter() {
            nodes.insert(edge.from);
            nodes.insert(edge.to);
        }
        let node_count: i32 = nodes.len() as i32;

        // we remove all nodes that are parent of some other
        // node. In end we are left with leaf nodes only.
        for edge in self.edges.iter() {
            nodes.remove(&edge.from);
        }
        let exit_count: i32 = nodes.len() as i32;

        edge_count - node_count + 2 * exit_count
    }
}

pub trait Parser {
    fn parse(&mut self, file: String) -> Graph;
}

pub fn calculate<T: Parser>(file: String, mut parser: T) -> i32 {
    let graph: Graph = parser.parse(file);
    graph.calculate_complexity()
}
