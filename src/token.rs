#[derive(Debug, PartialEq)]
pub enum TokenType {
    Newline,
    Tab,
    Space,
    Heading,
    Hash,
    Text,
    Error,
    Whitespace(char),
    Checkbutton(bool),
    ImageAlt,
    ImageSrc,
    LinkText,
    LinkHref,
    HorizontalRule,
    BlockquoteBegin,
    BlockquoteEnd,
    Code,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub id: TokenType,
    pub begin: usize,
    pub end: usize,
}

impl Token {
    pub fn new(id: TokenType, begin: usize, end: usize) -> Token {
        Token {
            id,
            begin,
            end,
        }
    }

    pub fn new_single(id: TokenType, begin: usize) -> Token {
        Token {
            id,
            begin,
            end: begin + 1,
        }
    }

    pub fn new_double(id: TokenType, begin: usize) -> Token {
        Token {
            id,
            begin,
            end: begin + 2,
        }
    }
}