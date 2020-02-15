use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::fmt::{self, Display, Formatter};
use std::cmp::min;

struct Node {
    children: Vec<Edge>,
    slink: Option<Weak<RefCell<Node>>>,
    is_root: bool,
    leaf_range: (usize, usize),
    id: u32
}

#[derive(Clone)]
struct Edge {
    node: Rc<RefCell<Node>>,
    character: u32,
    from: u32,
    len: u32
}

struct Infix {
    node: Rc<RefCell<Node>>,
    rest: Option<(u32, u32)>
}

pub struct SuffixTree {
    text: Vec<u32>,
    original_text: String,
    root: Rc<RefCell<Node>>,
    longest_non_leaf: Infix,
    leaf_dists: Vec<u32>,
    next_node_id: u32
}

impl Node {
    fn get_slink(&self) -> Rc<RefCell<Node>> {
        self.slink.as_ref().unwrap().upgrade().unwrap()
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    fn new(children: Vec<Edge>, is_root: bool, id: u32) -> Self {
        Self {
            children,
            is_root,
            slink: None,
            leaf_range: (0, 0),
            id
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut d = format!("node -> id = {}\n", self.id.to_string());
        d = format!("{}children:\n", d);
        for child in &self.children {
            d = format!("{}    child -> {}\n", d, child);
        }
        let slink = match &self.slink {
            None => String::from("()"),
            Some(s) => s.upgrade().unwrap().borrow().id.to_string()
        };
        d = format!("{}slink {}\n\n", d, slink);
        for child in &self.children {
            d = format!("{}{}\n", d, child.node.borrow());
        }
        write!(f, "{}", d)
    }
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "to {}, char {}, from {}, len {}", self.node.borrow().id, self.character as u8 as char, self.from, self.len)
    }
}

impl Display for SuffixTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.root.borrow())
    }
}

impl Display for Infix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.rest {
            None => write!(f, "infix -> node {}, no rest", self.node.borrow().id),
            Some((c, l)) => write!(f, "infix -> node {}, character {}, len {}", self.node.borrow().id, c as u8 as char, l)
        }
    }
}

impl SuffixTree {
    fn get_edge(&self, node: &Rc<RefCell<Node>>, character: u32) -> Option<Edge> {
        for edge in &node.borrow().children {
            if edge.character == character {
                let mut edge = edge.clone();
                if edge.node.borrow().is_leaf() {
                    edge.len = self.text.len() as u32 - edge.from;
                }
                return Some(edge);
            }
        }
        None
    }

    fn new_node(&mut self, children: Vec<Edge>, is_root: bool) -> Rc<RefCell<Node>> {
        self.next_node_id += 1;
        Rc::new(RefCell::new(Node::new(children, is_root, self.next_node_id)))
    }

    fn update_edge(node: &Rc<RefCell<Node>>, character: u32, new_edge: Edge) {
        let index = node.borrow().children.iter()
            .position(|x| x.character == character).unwrap();
        node.borrow_mut().children[index] = new_edge;
    }

    fn traverse_infix(
        &self,
        node: Rc<RefCell<Node>>,
        character: u32,
        from: u32,
        len: u32
    ) -> Infix {
        let edge = self.get_edge(&node, character).unwrap();
        if edge.len > len {
            Infix {
                node,
                rest: Some((edge.character, len))
            }
        } else if edge.len == len {
            Infix {
                node: edge.node,
                rest: None
            }
        } else {
            let next_char = self.text[(from + edge.len) as usize];
            self.traverse_infix(
                edge.clone().node,
                next_char,
                from + edge.len + 1,
                len - edge.len - 1
            )
        }
    }

    fn find_next_suffix(&self, suf: &Infix) -> Infix {
        match suf.rest {
            None => Infix {
                node: suf.node.borrow().get_slink(),
                rest: None
            },
            Some((character, len)) => {
                let edge = self.get_edge(&suf.node, character).unwrap();
                if suf.node.borrow().is_root && len > 0 {
                    let next_char = self.text[edge.from as usize];
                    self.traverse_infix(suf.node.clone(), next_char, edge.from + 1, len - 1)
                } else if suf.node.borrow().is_root && len == 0 {
                    Infix {
                        node: suf.node.clone(),
                        rest: None
                    }
                } else {
                    self.traverse_infix(
                        suf.node.borrow().get_slink(),
                        character,
                        edge.from,
                        len
                    )
                }
            }
        }
    }

    fn check_next_char(&self, inf: &Infix, character: u32) -> bool {
        if let Some((infix_char, len)) = inf.rest {
            let edge = self.get_edge(&inf.node, infix_char).unwrap();
            self.text[(edge.from + len) as usize] == character
        } else {
            self.get_edge(&inf.node, character).is_some()
        }
    }

    fn infix_plus_char(&self, inf: &Infix, character: u32) -> Option<Infix> {
        match inf.rest {
            None => {
                let edge = self.get_edge(&inf.node, character)?;
                if edge.len == 0 {
                    Some(Infix {
                        node: edge.node.clone(),
                        rest: None
                    })
                } else {
                    Some(Infix {
                        node: inf.node.clone(),
                        rest: Some((character, 0))
                    })
                }
            },
            Some((infix_char, len)) => {
                let edge = self.get_edge(&inf.node, infix_char).unwrap();
                if self.text[(edge.from + len) as usize] != character {
                    return None;
                }
                if edge.len == len + 1 {
                    Some(Infix {
                        node: edge.node,
                        rest: None
                    })
                } else {
                    Some(Infix {
                        node: inf.node.clone(),
                        rest: Some((infix_char, len + 1))
                    })
                }
            }
        }
    }

    fn add_character(&mut self, character: u32) {
        self.text.push(character);
        let mut last_inner_node: Option<Rc<RefCell<Node>>> = None;
        while !self.check_next_char(&self.longest_non_leaf, character) {
            let new_leaf = self.new_node(vec![], false);
            let edge_to_new_leaf = Edge {
                node: new_leaf,
                character,
                from: self.text.len() as u32,
                len: 0
            };
            match self.longest_non_leaf.rest {
                None => {
                    self.longest_non_leaf.node.borrow_mut().children.push(edge_to_new_leaf);
                    if let Some(node) = &last_inner_node {
                        node.borrow_mut().slink = 
                            Some(Rc::downgrade(&self.longest_non_leaf.node));
                        last_inner_node = None;
                    }
                }
                Some((infix_char, infix_len)) => {
                    let inner_edge = self.get_edge(
                        &self.longest_non_leaf.node,
                        infix_char
                    ).unwrap();
                    let inner_edge_end = Edge {
                        node: inner_edge.node,
                        character: self.text[(inner_edge.from + infix_len) as usize],
                        from: inner_edge.from + infix_len + 1,
                        len: inner_edge.len - infix_len - 1
                    };
                    let inner_node = self.new_node(
                        vec![edge_to_new_leaf, inner_edge_end],
                        false
                    );
                    let inner_edge_start = Edge {
                        node: inner_node.clone(),
                        character: inner_edge.character,
                        from: inner_edge.from,
                        len: infix_len
                    };
                    Self::update_edge(
                        &self.longest_non_leaf.node,
                        inner_edge.character,
                        inner_edge_start
                    );
                    if let Some(node) = &last_inner_node {
                        node.borrow_mut().slink = Some(Rc::downgrade(&inner_node));
                    }
                    last_inner_node = Some(inner_node);
                }
            }

            if self.longest_non_leaf.node.borrow().is_root &&
               self.longest_non_leaf.rest.is_none() {
                return;
            }
            self.longest_non_leaf = self.find_next_suffix(&self.longest_non_leaf)
        }

        if let Some(node) = last_inner_node {
            node.borrow_mut().slink = Some(Rc::downgrade(&self.longest_non_leaf.node));
        }
        self.longest_non_leaf = self.infix_plus_char(&self.longest_non_leaf, character).unwrap();
    }

    pub fn new(text: &str) -> Self {
        let node = Rc::new(RefCell::new(Node::new(vec![], true, 1)));
        let mut suf_tree = Self {
            text: Vec::with_capacity(text.len() + 1),
            original_text: String::from(text),
            root: node.clone(),
            longest_non_leaf: Infix {
                node: node.clone(),
                rest: None
            },
            leaf_dists: vec![],
            next_node_id: 1
        };

        for character in text.chars() {
            suf_tree.add_character(character as u32);   
        }
        suf_tree.add_character(123_456_789); // to make all sufixes leaves

        suf_tree.index_leaves(suf_tree.root.clone(), 0);

        suf_tree
    }

    fn index_leaves(&mut self, node: Rc<RefCell<Node>>, dist: u32) {
        let start = self.leaf_dists.len();
        if node.borrow().is_leaf() {
            self.leaf_dists.push(dist);
        }
        for child in &node.borrow().children {
            let child_dist = 1 + if child.node.borrow().is_leaf() {
                self.text.len() as u32 - child.from
            } else {
                child.len
            };
            self.index_leaves(child.node.clone(), dist + child_dist);
        }
        let end = self.leaf_dists.len();
        node.borrow_mut().leaf_range = (start, end);
    }

    pub fn search(&self, part: &str, context_len: u32) -> Vec<String> {
        let mut infix = Infix {
            node: self.root.clone(),
            rest: None
        };
        for character in part.chars() {
            infix = match self.infix_plus_char(&infix, character as u32) {
                None => return vec![],
                Some(r) => r
            }
        }

        let mut result: Vec<String> = vec![];
        let node = match infix.rest {
            None => infix.node,
            Some((infix_char, _)) => {
                let edge = self.get_edge(&infix.node, infix_char).unwrap();
                edge.node
            }
        };
        let (leaf_start, leaf_end) = node.borrow().leaf_range;
        let part_len = part.chars().count();
        for leaf_dist in &self.leaf_dists[leaf_start..leaf_end] {
            let from = self.text.len() - *leaf_dist as usize;
            let to = min(from + part_len + (context_len as usize), self.text.len() - 1);
            let s: String = self.original_text.chars().skip(from).take(to - from).collect();
            result.push(String::from(s));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equal_vectors<T: Eq>(v1: Vec<T>, v2: Vec<T>) -> bool {
        if v1.len() != v2.len() {
            return false;
        }
        for element in &v2 {
            if !v1.contains(element) {
                return false;
            }
        }
        for element in &v1 {
            if !v2.contains(element) {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_search_en() {
        let suffix_tree = SuffixTree::new("abcbc");
        assert!(equal_vectors(
            suffix_tree.search("bc", 1),
            vec![String::from("bc"), String::from("bcb")]
        ));
        assert!(equal_vectors(
            suffix_tree.search("abcbc", 1),
            vec![String::from("abcbc")]
        ));
        assert!(equal_vectors(
            suffix_tree.search("a", 0),
            vec![String::from("a")]
        ));
        assert!(equal_vectors(
            suffix_tree.search("", 1),
            vec![
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("b"),
                String::from("c"),
                String::from("")
            ]
        ));
        assert!(equal_vectors(
            suffix_tree.search("bcb", 10),
            vec![String::from("bcbc")]
        ));
    }

    #[test]
    fn test_search_bg() {
        let suffix_tree = SuffixTree::new("абвбвабб");
        assert!(equal_vectors(
            suffix_tree.search("б", 3),
            vec![
                String::from("бвбв"),
                String::from("бваб"),
                String::from("бб"),
                String::from("б")
            ]
        ));
        assert!(equal_vectors(
            suffix_tree.search("ваб", 3),
            vec![String::from("вабб")]
        ));
        assert!(equal_vectors(
            suffix_tree.search("ббб", 3),
            vec![]
        ));
    }
}
