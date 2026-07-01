#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Directive(Directive),
    Block(Block),
    Comment(String),
    BlankLine,
    OpaqueBlock(OpaqueBlock),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub leading_comments: Vec<String>,
    pub name: String,
    pub args: Vec<String>,
    pub inline_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub leading_comments: Vec<String>,
    pub name: String,
    pub args: Vec<String>,
    pub children: Vec<Node>,
    pub inline_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueBlock {
    pub leading_comments: Vec<String>,
    pub header: String,
    pub body: String,
    pub inline_comment: Option<String>,
}

impl Directive {
    pub fn new(name: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            leading_comments: Vec::new(),
            name: name.into(),
            args,
            inline_comment: None,
        }
    }
}

impl Block {
    pub fn new(name: impl Into<String>, args: Vec<String>, children: Vec<Node>) -> Self {
        Self {
            leading_comments: Vec::new(),
            name: name.into(),
            args,
            children,
            inline_comment: None,
        }
    }
}

impl OpaqueBlock {
    pub fn new(header: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            leading_comments: Vec::new(),
            header: header.into(),
            body: body.into(),
            inline_comment: None,
        }
    }
}

pub fn is_lua_block_name(name: &str) -> bool {
    name.ends_with("_by_lua_block")
}
