use std::collections::BTreeMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    In(String),
    Out(String),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::In(s) => write!(f, "In{{{}}}", s),
            Self::Out(s) => write!(f, "Out{{{}}}", s),
        }
    }
}

impl Action {
    /// return the opposite variant of the action
    pub fn bar(&self) -> Self {
        use Action::*;

        match self {
            In(s) => Out(s.clone()),
            Out(s) => In(s.clone()),
        }
    }

    /// turn the action into its opposite variant
    pub fn into_bar(self) -> Self {
        use Action::*;

        match self {
            In(s) => Out(s),
            Out(s) => In(s),
        }
    }

    pub fn channel(&self) -> String {
        use Action::*;

        match self {
            In(s) => s.to_owned(),
            Out(s) => s.to_owned(),
        }
    }

    /// return the same action over a new action
    pub fn with_new_channel(&self, new_channel: impl Into<String>) -> Self {
        use Action::*;

        match self {
            In(_) => In(new_channel.into()),
            Out(_) => Out(new_channel.into()),
        }
    }
}

pub type Map = BTreeMap<String, String>;

/// Nodes used in the syntax tree. The tree is generated by the parser: ['crate::parser::parse']
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Node {
    Recurse(String, Box<Node>),
    Restrict(Box<Node>, Action),
    Relabel(Box<Node>, Map),
    Compose(Box<Node>, Box<Node>),
    Choice(Box<Node>, Box<Node>),
    Prefix(Action, Box<Node>),
    Name(String), //leaf
    Nil,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", maxfmt(0, self))
    }
}

fn infix_recurse(node: &Node, s: &mut String) {
    use Node::*;

    match node {
        Recurse(string, node) => {
            s.push_str(&format!("_rec {}.", string));
            infix_recurse(node, s);
        }
        Restrict(node, action) => {
            infix_recurse(node, s);
            s.push_str(&format!("\\{}", action));
        }
        Relabel(node, map) => {
            infix_recurse(node, s);
            s.push_str(&format!("[{}]", print_map(map)));
        }
        Compose(node_a, node_b) => {
            s.push('(');
            infix_recurse(&node_a, s);
            s.push_str(" | ");
            infix_recurse(&node_b, s);
            s.push(')');
        }
        Choice(node_a, node_b) => {
            s.push('(');
            infix_recurse(&node_a, s);
            s.push_str(" + ");
            infix_recurse(&node_b, s);
            s.push(')');
        }
        Prefix(action, node) => {
            s.push_str(&format!("{}.", action));
            infix_recurse(node, s);
        }
        Name(string) => s.push_str(string),
        Nil => s.push_str("nil"),
    };
}

impl Node {
    /// Returns the syntax tree (a Node) in infix notation
    pub fn infix(&self) -> String {
        let mut s = String::new();

        infix_recurse(self, &mut s);
        s
    }
}

fn maxfmt(indent: u8, node: &Node) -> String {
    use Node::*;
    const TAB: &str = "   ";
    let tabs = TAB.repeat(indent as usize);
    let mut s = String::new();
    match node {
        Recurse(string, node) => {
            let node = maxfmt(indent + 1, node);
            s.push_str(&format!(
                "{tabs}Recurse(\n{tabs}{tab}{},\n{}\n{tabs})",
                string,
                node,
                tab = TAB,
                tabs = tabs
            ));
        }
        Restrict(node, action) => {
            let node = maxfmt(indent + 1, node);
            s.push_str(&format!(
                "{tabs}Restrict(\n{tabs}{tab}{},\n{}\n{tabs})",
                action,
                node,
                tab = TAB,
                tabs = tabs
            ));
        }
        Relabel(node, map) => {
            let node = maxfmt(indent + 1, node);
            s.push_str(&format!(
                "{tabs}Relabel(\n{tabs}{tab}{},\n{}\n{tabs})",
                print_map(map),
                node,
                tab = TAB,
                tabs = tabs
            ));
        }
        Compose(node_a, node_b) => {
            let node_a = maxfmt(indent + 1, node_a);
            let node_b = maxfmt(indent + 1, node_b);
            s.push_str(&format!(
                "{tabs}Compose(\n{},\n{}\n{tabs})",
                node_a,
                node_b,
                tabs = tabs
            ));
        }
        Choice(node_a, node_b) => {
            let node_a = maxfmt(indent + 1, node_a);
            let node_b = maxfmt(indent + 1, node_b);
            s.push_str(&format!(
                "{tabs}Choice(\n{},\n{}\n{tabs})",
                node_a,
                node_b,
                tabs = tabs
            ));
        }
        Prefix(action, node) => {
            let node = maxfmt(indent + 1, node);
            s.push_str(&format!(
                "{tabs}Prefix(\n{tabs}{tab}{},\n{}\n{tabs})",
                action,
                node,
                tab = TAB,
                tabs = tabs
            ));
        }
        Name(string) => {
            s.push_str(&format!("{tabs}Name(\"{}\")", string, tabs = tabs));
        }
        Nil => {
            s.push_str(&format!("{tabs}nil", tabs = tabs));
        }
    }
    s
}

fn print_map(map: &Map) -> String {
    let mut s = String::new();
    for (key, value) in map.iter() {
        s.push_str(&format!("{} → {}, ", key, value));
    }
    s
}
