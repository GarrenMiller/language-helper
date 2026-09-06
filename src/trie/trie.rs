// Need to build a byte prefix trie. This means a collection of Nodes, which are the postions of
// bytes. There are Edges between Nodes, and each Edge has an associated byte; hence the name.
//
// Nodes can be terminal, meaning they are the end of a path, the traversal of which results in a
// token. Terminal nodes have no children.
//
// Nodes that are not terminal and have no children are dead-end Nodes.
//
// There is the root Node, which has no parent and is not terminal. 
//
// Nodes only have one incoming edge, can have many outgoing
//
// Necessary functions:
// 1. Get valid next bytes for a given prefix.
// 2. Get all tokens that contain a prefix.
// 3. Add new token in and update paths accordingly.
//
// TODO: What happens when we have something like "car" and "cart"? "r" gets marked terminal, but
// then we need another node for non-terminal "r"
//
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use log::info;

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct Node {
    id: u64,
    value: u8,
    is_terminal: bool,
}

pub struct Trie {
    root_id: u64,
    nodes: HashMap<u64, Node>,
    children: HashMap<u64, Vec<u64>>
}

impl Node {
    pub fn new(byte: &u8, is_terminal: bool) -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed),
            value: *byte,
            is_terminal: is_terminal,
        }
    }
}


impl Trie {
    pub fn new() -> Self {
        let root = Node::new(&b'"', false);

        let mut instance = Self {
            root_id: root.id,
            nodes: HashMap::new(),
            children: HashMap::new(),
        };

        instance.nodes.insert(root.id, root);
        return instance;
    }

    pub fn path_token_difference(&self, path: Option<Vec<u64>>, token: &[u8]) -> Vec<u8> {
        let bytes = match path {
            Some(path) => &token[path.len()..],
            None => token,
        };
        return bytes.to_vec();
    }
    
    pub fn traverse_path(&self, path: &Vec<u64>) -> Vec<u8> {
        path.iter().map(|idx| self.nodes.get(idx).unwrap().value).collect::<Vec<u8>>()
    }

    pub fn add_node(&mut self, byte: u8, is_terminal: bool) -> &Node {
        let next_node = Node::new(&byte, is_terminal);
        self.nodes.entry(next_node.id).or_insert(next_node)
    }

    pub fn add_child(&mut self, parent_id: u64, child_id: u64) {
        self.children.entry(parent_id).or_insert_with(Vec::new).push(child_id);
    }

    pub fn add_token(&mut self, token: &[u8]) {
        let known_path = self.bytes_to_path(token);
        let mut parent_id = known_path.as_ref().and_then(|v| v.last().copied());
        let new_bytes = self.path_token_difference(known_path, token);
        let mut iter = new_bytes.iter().peekable();

        while let Some(byte) = iter.next() {
            let is_terminal = iter.peek().is_none();
            let node_id = self.add_node(*byte, is_terminal).id;
            match parent_id {
                Some(parent_id) => self.add_child(parent_id, node_id),
                None => (),
            };
            parent_id = Some(node_id);
        }
    }

    pub fn add_tokens(&mut self, tokens: &[&str]) {
        for token in tokens {
            self.add_token(token.as_bytes());
        }
    }

    fn bytes_to_path(&self, token: &[u8]) -> Option<Vec<u64>> {
        let mut current_node = &self.root_id; 
        let mut path = vec![];

        for byte in token {
            let children = self.children.get(current_node);
            match children {
                None => return (!path.is_empty()).then_some(path),
                Some(children) => {
                    let next_node = children
                        .iter()
                        .find(|id| &self.nodes.get(id).expect("All nodes must have an ID; something is wrong").value == byte);
                   
                    if let Some(value) = next_node {
                        path.push(*value);
                        current_node = value;
                        continue;
                    }
                }
            }
        }
        Some(path)
    }

    pub fn valid_next_nodes(&self, path: Vec<u64>) -> Option<&Vec<u64>> {
        if path.len() == 0 {
           return None; 
        }
        if !self.children.get(&self.root_id).unwrap().contains(&path[0]) {
            info!("Unrooted paths are not supported");
            return None;
        }
        info!("path is: {:?}", path);
        let result = self.children.get(&path[&path.len() - 1]);
        info!("Children are: {:?}", self.children);
        info!("result is :{:?}", result);
        result
    }

    pub fn valid_next_bytes(&self, prefix: &[u8]) -> Option<Vec<&u8>> {
        let root_children_bytes = self.children.get(&self.root_id)?
            .iter()
            .map(|id| self.nodes.get(id).unwrap().value)
            .collect::<Vec<u8>>();

        if !root_children_bytes.contains(&prefix[0]) {
            return None;
        }

        let path = self.bytes_to_path(prefix);
        match path {
            Some(path) => {
                let last_index = path.len().checked_sub(1);
                if let Some(last_index) = last_index {
                    return Some(self.children.get(&path[last_index])?.iter().map(|child_id| &self.nodes.get(child_id).unwrap().value).collect::<Vec<&u8>>())
                }
                return None;
            },
            None => None
        }
    }

    pub fn find_tokens_with_prefix(&self, prefix: &[u8]) -> Option<Vec<Vec<u8>>> {
        let path = self.bytes_to_path(prefix);
        match path {
            Some(root_path) => {
                let mut temp_token_paths = vec![root_path];
                let mut final_token_paths = Vec::new();

                while let Some(path) = temp_token_paths.pop() {
                    let last = path[path.len() - 1];
                    if self.nodes.get(&last)?.is_terminal {
                        final_token_paths.push(path);
                        continue;
                    }

                    let Some(next_nodes) = self.valid_next_nodes(path.clone()) else { info!("No valid next nodes"); return None };
                    for node in next_nodes {
                        let mut path_copy = path.clone();
                        path_copy.push(*node);
                        temp_token_paths.push(path_copy);
                    }
                }
                let mut tokens = Vec::new();
                for path in final_token_paths {
                    let token = path.iter().map(|index| self.nodes.get(index).unwrap().value).collect::<Vec<u8>>(); 
                    info!("Pushing token: {:?}", str::from_utf8(&token));
                    tokens.push(token);
                }
                Some(tokens)
            },
            None => None
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    fn distinct_prefixes(tokens: &[&str]) -> usize {
        let mut prefixes: Vec<&str> = Vec::new();

        for token in tokens {
            for (idx, _) in token .char_indices() {
                let prefix = &token[0..idx + 1];
                if !prefixes.contains(&prefix) {
                    prefixes.push(prefix)
                }; 
            }
        }
        prefixes.len()
    }

    #[test]
    fn test_distinct_prefixes() {
        let input = ["creative", "creature"];
        let output = distinct_prefixes(&input);
        assert_eq!(output, 11);
    }

    #[test]
    fn test_no_duplicate_nodes() {
        let mut trie = Trie::new();
        let input = ["car", "cart", "carts"];
        let num_distinct_prefixes = distinct_prefixes(&input);
        trie.add_tokens(&input);

        assert_eq!(trie.nodes.len(), 1 + num_distinct_prefixes); // All nodes are unique 
    }

    #[test]
    fn test_terminal_nodes_exist_and_can_have_children() {
        let mut trie = Trie::new();
        trie.add_tokens(&["car", "cart", "carts"]);
        let terminal_nodes = trie.nodes.values().filter(|n| n.is_terminal).collect::<Vec<&Node>>();
        let terminal_nodes_with_children = terminal_nodes.iter().filter(|n| trie.children.get(&n.id).is_some()).collect::<Vec<&&Node>>();
        let mut node_values = terminal_nodes.iter().map(|n| n.value).collect::<Vec<u8>>();

        assert_eq!(terminal_nodes_with_children.len(), 2); 
        assert_eq!(terminal_nodes.len(), 3); 
        assert_eq!(node_values.sort(), vec![ b'r', b't', b's'].sort()); 
    }

    #[test]
    fn test_bytes_to_path() {
       let mut trie = Trie::new();
       trie.add_token(b"cart");
       let path = trie.bytes_to_path("carts".as_bytes()).unwrap();
       let prefix = trie.traverse_path(&path);
       assert_eq!(prefix.as_slice(), b"cart");
    }

    #[test]
    fn test_valid_next_nodes() {
        let mut trie = Trie::new();
        trie.add_tokens(&["bat", "board", "body", "bottom"]);
        let path = trie.bytes_to_path("bo".as_bytes()).unwrap();
        let next_nodes = trie.valid_next_nodes(path).unwrap();

        assert_eq!(next_nodes.len(), 3);
    }

    #[test]
    fn test_valid_next_bytes() {
        let mut trie = Trie::new();
        trie.add_token(b"cards");
        trie.add_token(b"cardiology");
        let mut next_bytes = trie.valid_next_bytes(b"card").unwrap();
        assert_eq!(next_bytes.sort(), vec![b'i', b's'].sort());
    }

    #[test]
    fn test_tokens_with_prefix() {
        let mut trie = Trie::new();
        trie.add_token(b"bat");
        trie.add_token(b"board");
        trie.add_token(b"body");
        trie.add_token(b"bottom");
        let tokens = trie.find_tokens_with_prefix(b"bo");
        assert_eq!(tokens.is_none(), false);
        assert_eq!(tokens.unwrap(), vec![b"board".to_vec(), b"body".to_vec(), b"bottom".to_vec()]);
    }
}
