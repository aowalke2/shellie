use std::collections::HashMap;

#[derive(Default, Debug)]
struct TrieNode {
    end_of_word: bool,
    children: HashMap<char, TrieNode>,
}

#[derive(Default, Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for c in word.chars() {
            node = node.children.entry(c).or_default()
        }
        node.end_of_word = true;
    }

    pub fn search(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;
        let mut suggestions = Vec::new();
        for c in prefix.chars() {
            match node.children.get(&c) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }

        self.get_suggestions(node, prefix, &mut suggestions);
        suggestions
    }

    fn get_suggestions(&self, node: &TrieNode, prefix: &str, suggestions: &mut Vec<String>) {
        if node.end_of_word {
            suggestions.push(prefix.to_string());
        }

        for (c, child) in &node.children {
            let prefix = format!("{}{}", prefix, c);
            self.get_suggestions(child, &prefix, suggestions);
        }
    }
}

pub fn build_trie() -> Trie {
    let mut trie = Trie::new();
    trie.insert("echo");
    trie.insert("exit");
    trie
}
