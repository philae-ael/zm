use crate::parser::Value;

#[derive(Debug, Clone, Copy, Default)]
pub struct Loc {
    pub column: usize,
    pub line: usize,
}

impl Loc {
    fn advance(&mut self, c: char) {
        match c {
            '\n' => {
                self.line += 1;
                self.column = 0;
            }
            _ => {
                self.column += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocRange {
    pub start: Loc,
    pub end: Loc,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Eof,
    Ident(&'a str),
    Value(Value),
}

impl Token<'_> {
    pub fn to_owned(&self) -> TokenOwned {
        match self {
            Token::Eof => TokenOwned::Eof,
            Token::Ident(ident) => TokenOwned::Ident(ident.to_string()),
            Token::Value(v) => TokenOwned::Value(v.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenOwned {
    Eof,
    Ident(String),
    Value(Value),
}

#[derive(Debug, Clone, Copy)]
pub enum TokenName {
    Eof,
    Ident,
}

pub struct Tokenizer<'a> {
    reader: &'a mut dyn std::io::Read,
    buffer: Vec<u8>,
    offset: usize,
    eof: bool,
    token: String,
    loc: Loc,
}

impl<'a> Tokenizer<'a> {
    pub fn new(reader: &'a mut dyn std::io::Read) -> Self {
        Self {
            reader,
            buffer: Vec::with_capacity(512),
            eof: false,
            offset: 0,
            token: String::new(),
            loc: Loc::default(),
        }
    }

    fn fill_buf(&mut self) {
        self.buffer.resize(self.buffer.capacity(), 0);
        let amount = self.reader.read(&mut self.buffer).unwrap();
        self.buffer.truncate(amount);

        if amount == 0 {
            self.eof = true;
        }

        self.offset = 0;
    }

    fn next_char(&mut self, advance: bool) -> Option<(char, Loc)> {
        if self.eof {
            return None;
        }

        if self.buffer.len() == self.offset {
            self.fill_buf();
            if self.eof {
                return None;
            }
        }

        let available = &self.buffer[self.offset..];

        // We only support ascii as of now
        char::try_from(available[0] as u32)
            .ok()
            .map(|c| (c, self.advance(c, advance)))
    }

    fn advance(&mut self, c: char, advance: bool) -> Loc {
        if advance {
            self.offset += c.len_utf8();
            self.loc.advance(c);
        }
        return self.loc;
    }

    fn whitespace(&mut self, advance: bool) -> Option<(char, Loc)> {
        match self.next_char(false)? {
            (c @ (' ' | '\t' | '\n' | '\r'), _) => Some((c, self.advance(c, advance))),
            _ => None,
        }
    }

    fn not_whitespace(&mut self, advance: bool) -> Option<(char, Loc)> {
        match self.whitespace(false) {
            Some(_) => None,
            None => self.next_char(advance),
        }
    }

    pub fn next_token(&mut self) -> anyhow::Result<(Token, LocRange)> {
        self.token.clear();

        while self.whitespace(true).is_some() {}

        let start = self.loc;
        let mut end = start;
        while let Some((c, loc)) = self.not_whitespace(true) {
            end = loc;
            self.token.push(c);
        }

        Ok((
            if self.eof {
                Token::Eof
            } else {
                Token::Ident(&self.token)
            },
            LocRange { start, end },
        ))
    }

    pub fn read_line(&mut self) -> anyhow::Result<(Token, LocRange)> {
        self.token.clear();

        let start = self.loc;
        let mut end = start;
        while let Some((c, loc)) = self.next_char(true) {
            match c {
                '\n' => {
                    break;
                }
                c => {
                    end = loc;
                    self.token.push(c)
                }
            }
        }

        Ok((
            if self.eof {
                Token::Eof
            } else {
                Token::Value(Value::String(std::mem::take(&mut self.token)))
            },
            LocRange { start, end },
        ))
    }

    pub fn read_string(&mut self) -> anyhow::Result<(Token, LocRange)> {
        self.token.clear();

        while self.whitespace(true).is_some() {}

        let in_quote = matches!(self.next_char(false), Some(('"', _)));
        if in_quote {
            self.next_char(true);
        }

        let start = self.loc;
        let mut end = start;
        while let Some((c, loc)) = if in_quote {
            self.next_char(true)
        } else {
            self.not_whitespace(true)
        } {
            if in_quote && c == '"' {
                break;
            }

            end = loc;
            self.token.push(c)
        }

        Ok((
            if self.eof {
                Token::Eof
            } else {
                Token::Value(Value::String(std::mem::take(&mut self.token)))
            },
            LocRange { start, end },
        ))
    }

    pub fn read_num(&mut self) -> anyhow::Result<(Token, LocRange)> {
        self.token.clear();

        while self.whitespace(true).is_some() {}
        let start = self.loc;
        let mut end = start;
        while let Some((c, loc)) = self.next_char(true) {
            match c {
                '0'..='9' => {
                    end = loc;
                    self.token.push(c)
                }
                _ => {
                    break;
                }
            }
        }

        Ok((
            if self.eof {
                Token::Eof
            } else {
                Token::Value(Value::Num(std::mem::take(&mut self.token).parse().unwrap()))
            },
            LocRange { start, end },
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::Token;

    use super::Tokenizer;

    #[test]
    fn token() {
        let buf = " test tkoen 1234";
        let mut cursor = std::io::Cursor::new(buf.as_bytes());

        let mut t = Tokenizer::new(&mut cursor);

        let (ret, _) = t.next_token().unwrap();
        assert_eq!(ret, Token::Ident("test"));

        let (ret, _) = t.next_token().unwrap();
        assert_eq!(ret, Token::Ident("tkoen"));

        let (ret, _) = t.next_token().unwrap();
        assert_eq!(ret, Token::Ident("1234"));

        let (ret, _) = t.next_token().unwrap();
        assert_eq!(ret, Token::Eof);
    }
}
