use crate::lexer::token::Token;

#[derive(Debug)]
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.pos < self.source.len() {
            self.skip_whitespace();

            if self.pos >= self.source.len() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() && self.current().is_whitespace() {
            self.advance();
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let c = self.current();

        match c {
            '(' => {
                self.advance();
                Ok(Token::LParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RParen)
            }
            '{' => {
                self.advance();
                Ok(Token::LBrace)
            }
            '}' => {
                self.advance();
                Ok(Token::RBrace)
            }
            '[' => {
                self.advance();
                Ok(Token::LBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RBracket)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            ':' => {
                self.advance();
                Ok(Token::Colon)
            }
            '?' => {
                self.advance();
                Ok(Token::Question)
            }
            '%' => {
                self.advance();
                Ok(Token::Percent)
            }

            '+' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::PlusEq)
                } else if self.match_char('+') {
                    Ok(Token::PlusPlus)
                } else {
                    Ok(Token::Plus)
                }
            }
            '-' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::MinusEq)
                } else if self.match_char('-') {
                    Ok(Token::MinusMinus)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '/' => {
                if self.peek() == '/' {
                    self.skip_line_comment();
                    self.skip_whitespace();
                    self.next_token()
                } else if self.peek() == '*' {
                    self.skip_block_comment()?;
                    self.skip_whitespace();
                    self.next_token()
                } else {
                    self.advance();
                    Ok(Token::Slash)
                }
            }

            '=' => {
                self.advance();
                if self.match_char('=') {
                    if self.match_char('=') {
                        Ok(Token::EqEqEq)
                    } else {
                        Ok(Token::EqEq)
                    }
                } else {
                    Ok(Token::Eq)
                }
            }
            '!' => {
                self.advance();
                if self.match_char('=') {
                    if self.match_char('=') {
                        Ok(Token::BangEqEq)
                    } else {
                        Ok(Token::BangEq)
                    }
                } else {
                    Ok(Token::Bang)
                }
            }
            '<' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::LtEq)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                self.advance();
                if self.match_char('=') {
                    Ok(Token::GtEq)
                } else {
                    Ok(Token::Gt)
                }
            }
            '&' => {
                self.advance();
                if self.match_char('&') {
                    Ok(Token::AmpAmp)
                } else {
                    Err("Unexpected character '&'".into())
                }
            }
            '|' => {
                self.advance();
                if self.match_char('|') {
                    Ok(Token::PipePipe)
                } else {
                    Err("Unexpected character '|'".into())
                }
            }
            '.' => {
                self.advance();
                Ok(Token::Dot)
            }

            '"' | '\'' => self.read_string(c),
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' | '$' => self.read_ident(),

            _ => Err(format!("Unexpected character '{}'", c)),
        }
    }

    fn read_string(&mut self, quote: char) -> Result<Token, String> {
        self.advance(); // consume opening quote
        let mut s = String::new();

        while self.pos < self.source.len() {
            let c = self.current();
            if c == quote {
                self.advance();
                return Ok(Token::Str(s));
            }
            if c == '\\' {
                self.advance();
                let escaped = match self.current() {
                    'n' => '\n',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    _ => self.current(),
                };
                s.push(escaped);
            } else {
                s.push(c);
            }
            self.advance();
        }

        Err("Unterminated string".into())
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let start = self.pos;
        let mut has_dot = false;

        while self.pos < self.source.len() {
            let c = self.current();
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let s: String = self.source[start..self.pos].iter().collect();
        let n: f64 = s.parse().map_err(|_| format!("Invalid number: {}", s))?;
        Ok(Token::Number(n))
    }

    fn read_ident(&mut self) -> Result<Token, String> {
        let start = self.pos;

        while self.pos < self.source.len() {
            let c = self.current();
            if c.is_alphanumeric() || c == '_' || c == '$' {
                self.advance();
            } else {
                break;
            }
        }

        let s: String = self.source[start..self.pos].iter().collect();
        Ok(match s.as_str() {
            "let" => Token::Let,
            "const" => Token::Const,
            "var" => Token::Var,
            "function" => Token::Func,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "undefined" => Token::Undefined,
            _ => Token::Ident(s),
        })
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.source.len() && self.current() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        self.advance(); // /
        self.advance(); // *
        while self.pos < self.source.len() {
            if self.current() == '*' && self.peek() == '/' {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err("Unterminated block comment".into())
    }

    fn current(&self) -> char {
        self.source[self.pos]
    }

    fn peek(&self) -> char {
        if self.pos + 1 < self.source.len() {
            self.source[self.pos + 1]
        } else {
            '\0'
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn match_char(&mut self, c: char) -> bool {
        if self.pos < self.source.len() && self.source[self.pos] == c {
            self.advance();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<Token> {
        Lexer::new(source).tokenize().unwrap()
    }

    #[test]
    fn test_numbers() {
        let tokens = lex("42 3.14");
        assert_eq!(tokens[0], Token::Number(42.0));
        assert_eq!(tokens[1], Token::Number(3.14));
        assert_eq!(tokens[2], Token::Eof);
    }

    #[test]
    fn test_strings() {
        let tokens = lex("\"hello\" 'world'");
        assert_eq!(tokens[0], Token::Str("hello".into()));
        assert_eq!(tokens[1], Token::Str("world".into()));
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("foo bar_baz $el");
        assert_eq!(tokens[0], Token::Ident("foo".into()));
        assert_eq!(tokens[1], Token::Ident("bar_baz".into()));
        assert_eq!(tokens[2], Token::Ident("$el".into()));
    }

    #[test]
    fn test_keywords() {
        let tokens =
            lex("let const var function if else while for return true false null undefined");
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Const);
        assert_eq!(tokens[2], Token::Var);
        assert_eq!(tokens[3], Token::Func);
        assert_eq!(tokens[4], Token::If);
        assert_eq!(tokens[5], Token::Else);
        assert_eq!(tokens[6], Token::While);
        assert_eq!(tokens[7], Token::For);
        assert_eq!(tokens[8], Token::Return);
        assert_eq!(tokens[9], Token::True);
        assert_eq!(tokens[10], Token::False);
        assert_eq!(tokens[11], Token::Null);
        assert_eq!(tokens[12], Token::Undefined);
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / % == === != !== < > <= >= && || !");
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Percent);
        assert_eq!(tokens[5], Token::EqEq);
        assert_eq!(tokens[6], Token::EqEqEq);
        assert_eq!(tokens[7], Token::BangEq);
        assert_eq!(tokens[8], Token::BangEqEq);
        assert_eq!(tokens[9], Token::Lt);
        assert_eq!(tokens[10], Token::Gt);
        assert_eq!(tokens[11], Token::LtEq);
        assert_eq!(tokens[12], Token::GtEq);
        assert_eq!(tokens[13], Token::AmpAmp);
        assert_eq!(tokens[14], Token::PipePipe);
        assert_eq!(tokens[15], Token::Bang);
    }

    #[test]
    fn test_punctuation() {
        let tokens = lex("( ) { } [ ] , ; . : ?");
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::RParen);
        assert_eq!(tokens[2], Token::LBrace);
        assert_eq!(tokens[3], Token::RBrace);
        assert_eq!(tokens[4], Token::LBracket);
        assert_eq!(tokens[5], Token::RBracket);
        assert_eq!(tokens[6], Token::Comma);
        assert_eq!(tokens[7], Token::Semicolon);
        assert_eq!(tokens[8], Token::Dot);
        assert_eq!(tokens[9], Token::Colon);
        assert_eq!(tokens[10], Token::Question);
    }

    #[test]
    fn test_assignment_and_increment() {
        let tokens = lex("= ++ -- += -=");
        assert_eq!(tokens[0], Token::Eq);
        assert_eq!(tokens[1], Token::PlusPlus);
        assert_eq!(tokens[2], Token::MinusMinus);
        assert_eq!(tokens[3], Token::PlusEq);
        assert_eq!(tokens[4], Token::MinusEq);
    }

    #[test]
    fn test_line_comment() {
        let tokens = lex("42 // this is a comment\n99");
        assert_eq!(tokens[0], Token::Number(42.0));
        assert_eq!(tokens[1], Token::Number(99.0));
    }

    #[test]
    fn test_block_comment() {
        let tokens = lex("42 /* comment */ 99");
        assert_eq!(tokens[0], Token::Number(42.0));
        assert_eq!(tokens[1], Token::Number(99.0));
    }

    #[test]
    fn test_expression() {
        let tokens = lex("let x = 1 + 2 * 3;");
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Ident("x".into()));
        assert_eq!(tokens[2], Token::Eq);
        assert_eq!(tokens[3], Token::Number(1.0));
        assert_eq!(tokens[4], Token::Plus);
        assert_eq!(tokens[5], Token::Number(2.0));
        assert_eq!(tokens[6], Token::Star);
        assert_eq!(tokens[7], Token::Number(3.0));
        assert_eq!(tokens[8], Token::Semicolon);
    }

    #[test]
    fn test_unterminated_string() {
        let result = Lexer::new("\"hello").tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_unexpected_char() {
        let result = Lexer::new("@").tokenize();
        assert!(result.is_err());
    }
}
