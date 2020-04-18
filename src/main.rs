#![allow(dead_code, unused_imports, unused_variables)]

use std::fs;
use std::io;
use std::io::Write;
use std::iter;
use std::str;

mod position;
use position::Position;

mod token;
use token::Token;
use token::TokenType;

fn main() {
    let file: String = match read_markdown_file(String::from("test/heading.md")) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    let tokens = lex(&file).unwrap();
    let mut log = fs::File::create("log/heading.log").unwrap();
    log.write(format!("{:#?}", tokens).as_bytes()).unwrap();
    let html = parse(&file, &tokens);
    generate_html(String::from("generated_html/heading.html"), html);
}

fn read_markdown_file(file_path: String) -> Result<String, io::Error> {
    let file: String = fs::read_to_string(file_path)?;

    Ok(file)
}

fn lex(text: &String) -> Option<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::with_capacity(text.len());
    let mut iter = text.chars().enumerate();
    let mut pos: Position = Position::new(0, 0, 0);
    loop {
        match iter.next() {
            Some(c) => {
                pos.increment();
                match c.1 {
                    '#' => {
                        match_heading(&mut tokens, &mut iter, &mut pos, c);
                    },
                    '\n' => tokens.push(Token::new_single(TokenType::Newline, c.0)),
                    _ => tokens.push(Token::new_single(TokenType::Text, c.0)),
                }
            },
            None => break,
        }
    }

    Some(tokens)
}

fn match_heading(tokens: &mut Vec<Token>, iter: &mut iter::Enumerate<str::Chars>, pos: &mut Position, c: (usize, char)) {
    let mut heading_count: usize = 1;
    while let Some(v) = iter.next() {
        pos.increment();
        match v.1 {
            '#' => {
                heading_count += 1;
            },
            ' ' => {
                if heading_count > 6 {
                    tokens.push(Token::new(TokenType::Error, c.0, c.0 + heading_count));
                } else {
                    tokens.push(Token::new(TokenType::Heading, c.0, c.0 + heading_count));
                }
                tokens.push(Token::new_single(TokenType::Space, c.0 + heading_count));
                break;
            },
            _ =>  {
                loop  {
                    match iter.next() {
                        Some(v) => {
                            pos.increment();
                            match v.1 {
                                ' '|'\t'|'\n' => {
                                    tokens.push(Token::new(TokenType::Text, c.0, v.0));
                                    tokens.push(Token::new_single(TokenType::Whitespace(v.1), v.0));
                                    break;
                                },
                                _ => (),
                            }
                        },
                        None => {
                            tokens.push(Token::new(TokenType::Text, c.0, pos.index));
                            break;
                        },
                    }
                }
                break;
            },
        }
    }
}

fn parse(file: &String, tokens: &Vec<Token>) -> Vec<String> {
    let mut html: Vec<String> = Vec::with_capacity(file.len());
    let mut iter = tokens.iter().peekable();
    while let Some(t) = iter.next() {
        match t.id {
            TokenType::Heading => {
                let begin: usize = iter.next().unwrap().end;
                let mut end: usize = begin;
                while let Some(tok) = iter.peek() {
                    match tok.id {
                        TokenType::Text => {
                            end = tok.end;
                            iter.next();
                        },
                        _ => break,
                    }
                }
                html.push(format!("<h{}>{}</h{}>\n", t.end - t.begin, file[begin..end].to_string(), t.end - t.begin));
                if iter.peek().unwrap().id == TokenType::Newline {
                    iter.next();
                }
            },
            TokenType::Error => html.push(format!("<span class=\"error\">ERROR: {}</span>", file[t.begin..t.end].to_string())),
            TokenType::Newline => html.push("<br>\n".to_string()),
            TokenType::Text => html.push(file[t.begin..t.end].to_string()),
            TokenType::Space => html.push(file[t.begin..t.end].to_string()),
            TokenType::Whitespace(char) => html.push(file[t.begin..t.end].to_string()),
            _ => (),
        }
    }

    html
}

fn generate_html(output_file: String, html: Vec<String>) {
    let mut file = fs::File::create(output_file).unwrap();
    file.write("<link rel=\"stylesheet\" href=\"default.css\">\n".as_bytes()).unwrap();
    file.write("<div class=\"markdown-body\">\n".as_bytes()).unwrap();
    for tag in html.iter() {
        file.write(tag.as_bytes()).unwrap();
    }
    file.write("\n</div>".as_bytes()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_count() {
        let t = lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap();
        let mut headings: usize = 0;
        for token in t.iter() {
            if token.id == TokenType::Heading {
                headings += 1;
            } 
        }
        assert!(headings == 6);
    }
    
    #[test]
    #[should_panic]
    fn heading_count_fail() {
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().len() == 1);
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().len() == 10);
    }

    #[test]
    fn heading_begin() {
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().get(0).unwrap().begin == 0);
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().get(2).unwrap().begin == 25);
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().get(5).unwrap().begin == 70);
    }
    
    #[test]
    fn heading_end() {
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().get(0).unwrap().end == 1);
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().get(2).unwrap().end == 28);
        assert!(lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap().get(5).unwrap().end == 76);
    }
}
