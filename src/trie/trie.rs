// Need to build a byte prefix trie. This means a collection of Nodes, which contain prefix values. There
// are Edges between Nodes, and each Edge has an associated byte; hence the name.
//
// Nodes can be terminal, meaning they are the end of a path, the traversal of which results in a
// token. Terminal nodes have no children.
//
// Nodes that are not terminal and have no children are dead-end Nodes.
//
// There is the root Node, which has no parent and is not terminal. 
use uuid::Uuid;

struct Node {
    prefix: &[u8],
    parent: Option<Node>,
    terminal: bool,
}

struct Edge {
    value: u8,
    source: usize,
    target: usize,
}

struct Trie {
    nodes: Vec<Node>,
    edges: Vec<Edge>
}

impl Trie {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new()
        }
    }
}

fn get_token_nodes_and_edges(token: &str) {
    let tokens = Vec::new();
    for byte in token.bytes() {
        let id = Uuid::new_v4();
        tokens.push(Node { prefix: byte, parent: }); 
    }
}
