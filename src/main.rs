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
    let test_files = [
        "heading",
        "checkbutton",
        ];
    for test in test_files.iter() {
        let file: String = fs::read_to_string(format!("test/{}.md", test)).unwrap();
        let tokens = lex(&file).unwrap();
        let mut log = fs::File::create(format!("log/{}.log", test)).unwrap();
        log.write(format!("{:#?}", tokens).as_bytes()).unwrap();
        let html = parse(&file, &tokens);
        generate_html(format!("generated_html/{}.html", test), html);
    }
}

fn lex(text: &String) -> Option<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::with_capacity(text.len());
    let mut iter = text.chars().enumerate().peekable();
    let mut pos: Position = Position::new(0, 0, 0);
    loop {
        match iter.next() {
            Some(c) => {
                pos.increment();
                match c.1 {
                    '#' => {
                        match_heading(&mut tokens, &mut iter, &mut pos, c);
                    },
                    '-' => {
                        match_checkbutton(text, &mut tokens, &mut iter, &mut pos, c);
                    },
                    '\n' => tokens.push(Token::new_single(TokenType::Newline, c.0)),
                    '\t' => tokens.push(Token::new_single(TokenType::Tab, c.0)),
                    ' ' => tokens.push(Token::new_single(TokenType::Space, c.0)),
                    _ => tokens.push(Token::new_single(TokenType::Text, c.0)),
                }
            },
            None => break,
        }
    }

    Some(tokens)
}

fn match_heading(tokens: &mut Vec<Token>, iter: &mut iter::Peekable<iter::Enumerate<str::Chars>>, pos: &mut Position, c: (usize, char)) {
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

fn match_checkbutton(text: &String, tokens: &mut Vec<Token>, iter: &mut iter::Peekable<iter::Enumerate<str::Chars>>, pos: &mut Position, c: (usize, char)) {
    match text.get(c.0 + 1..c.0 + 6) {
        Some(v) => {
            if v == " [ ] " {
                tokens.push(Token::new(TokenType::Checkbutton(false), c.0, c.0 + 5));
                iter.nth(3);
            } else if v == " [x] " {
                tokens.push(Token::new(TokenType::Checkbutton(true), c.0, c.0 + 5));
                iter.nth(3);
            } else {
                tokens.push(Token::new_single(TokenType::Text, c.0));
            }
        },
        None => (),
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
                        TokenType::Text|TokenType::Space => {
                            end = tok.end;
                            iter.next();
                        },
                        _ => break,
                    }
                }
                html.push(format!("<h{}>{}</h{}>\n", t.end - t.begin, file[begin..end].to_string(), t.end - t.begin));
                match iter.peek() {
                    Some(n) => {
                        if n.id == TokenType::Newline {
                            iter.next();
                        }
                    },
                    None => (),
                }
            },
            TokenType::Checkbutton(bool) => {
                if t.id == TokenType::Checkbutton(true) {
                    html.push(format!("<input type=\"checkbox\" checked>"));
                } else {
                    html.push(format!("<input type=\"checkbox\">"));
                }
            },
            TokenType::Error => html.push(format!("<span class=\"error\">ERROR: {}</span>", file[t.begin..t.end].to_string())),
            TokenType::Newline => html.push("<br>\n".to_string()),
            TokenType::Text => html.push(file[t.begin..t.end].to_string()),
            TokenType::Space => html.push(file[t.begin..t.end].to_string()),
            TokenType::Tab => html.push(file[t.begin..t.end].to_string()),
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
    fn heading() {
        let t = lex(&fs::read_to_string("test/heading.md").unwrap()).unwrap();
        let mut headings: usize = 0;
        let mut errors: usize = 0;
        for token in t.iter() {
            match token.id {
                TokenType::Heading => {
                    headings += 1;
                },
                TokenType::Error => {
                    errors += 1;
                },
                TokenType::Text|TokenType::Space|TokenType::Newline|TokenType::Whitespace(' ') => (),
                _ => panic!("Encounterd TokenType other than expected!"),
            }
        }
        assert!(headings == 6);
        assert!(errors == 1);
    }

    #[test]
    fn checkbutton() {
        let t = lex(&fs::read_to_string("test/checkbutton.md").unwrap()).unwrap();
        let mut checkbuttons: usize = 0;
        for token in t.iter() {
            match token.id {
                TokenType::Checkbutton(bool) => {
                    checkbuttons += 1;
                },
                TokenType::Text|TokenType::Space|TokenType::Newline|TokenType::Whitespace(' ') => (),
                _ => panic!("Encounterd TokenType other than expected!"),
            }
        }
        assert!(checkbuttons == 2);
    }
}
