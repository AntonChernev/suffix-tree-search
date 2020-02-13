use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::fmt::{self, Display, Formatter};
use std::cmp::min;

struct Node {
    children: Vec<Edge>,
    slink: Option<Weak<RefCell<Node>>>,
    is_leaf: bool,
    is_root: bool,
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
    id: u32
}

impl Node {
    fn get_slink(&self) -> Rc<RefCell<Node>> {
        self.slink.as_ref().unwrap().upgrade().unwrap()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut d = format!("node -> id = {}, is leaf {}\n", self.id.to_string(), self.is_leaf);
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
                if edge.node.borrow().is_leaf {
                    edge.len = self.text.len() as u32 - edge.from;
                }
                return Some(edge);
            }
        }
        None
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
        // println!("add character {}", character as u8 as char);
        // println!("suffix {}", self.longest_non_leaf);
        self.text.push(character);
        let mut last_inner_node: Option<Rc<RefCell<Node>>> = None;
        while !self.check_next_char(&self.longest_non_leaf, character) {
            // println!("suffix {}", self.longest_non_leaf);
            let new_leaf = Rc::new(RefCell::new(Node {
                children: vec![],
                slink: None,
                is_leaf: true,
                is_root: false,
                id: self.id
            }));
            self.id += 1;
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
                    let inner_node = Rc::new(RefCell::new(Node {
                        children: vec![edge_to_new_leaf, inner_edge_end],
                        slink: None,
                        is_leaf: false,
                        is_root: false,
                        id: self.id
                    }));
                    self.id += 1;
                    let inner_edge_start = Edge {
                        node: inner_node.clone(),
                        character: inner_edge.character,
                        from: inner_edge.from,
                        len: infix_len
                    };
                    SuffixTree::update_edge(
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
        // println!("out of while suffix {}", self.longest_non_leaf);

        if let Some(node) = last_inner_node {
            node.borrow_mut().slink = Some(Rc::downgrade(&self.longest_non_leaf.node));
        }
        self.longest_non_leaf = self.infix_plus_char(&self.longest_non_leaf, character).unwrap();
    }

    pub fn new(text: &str) -> Self {
        let node = Rc::new(RefCell::new(Node {
            children: vec![],
            slink: None,
            is_leaf: false,
            is_root: true,
            id: 1
        }));
        let mut suf_tree = SuffixTree {
            text: Vec::with_capacity(text.len()),
            original_text: String::from(text),
            root: node.clone(),
            longest_non_leaf: Infix {
                node: node.clone(),
                rest: None
            },
            id: 2
        };
        for character in text.chars() {
            suf_tree.add_character(character as u32);   
        }

        suf_tree
    }

    pub fn search(&self, part: &str) -> Option<&str> {
        let mut infix = Infix {
            node: self.root.clone(),
            rest: None
        };
        for character in part.chars() {
            infix = self.infix_plus_char(&infix, character as u32)?;
        }

        let part_end = match infix.rest {
            Some((infix_char, len)) => {
                let edge = self.get_edge(&infix.node, infix_char).unwrap();
                edge.from + len
            },
            None => {
                if infix.node.borrow().is_leaf {
                    self.text.len() as u32
                } else {
                    let edge = &infix.node.borrow().children[0];
                    edge.from - 1
                }
            }
        };
        let from = part_end as usize - part.len();
        let to = min((part_end + 20) as usize, self.text.len());

        Some(&self.original_text[from..to])
    }
}
