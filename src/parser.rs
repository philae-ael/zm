use crate::tokenizer::{LocRange, Token, TokenName, TokenOwned, Tokenizer};

const ADD_UCODE: [UOps; 6] = [
    UOps::Push(Value::Num(0)),
    UOps::ReadNum,
    UOps::JmpRelZ(2),
    UOps::Add,
    UOps::JmpRel(-4),
    UOps::Print,
];
const MULL_UCODE: [UOps; 6] = [
    UOps::Push(Value::Num(1)),
    UOps::ReadNum,
    UOps::JmpRelZ(2),
    UOps::Mul,
    UOps::JmpRel(-4),
    UOps::Print,
];

const JOIN_UCODE: [UOps; 20] = [
    UOps::ReadString,
    UOps::Pop,
    // Drop until line end
    UOps::ReadLine,
    UOps::JmpRelZ(3),
    UOps::Pop,
    // Start from empty + do not join on first value
    UOps::Push(Value::String(String::new())),
    UOps::Dup,
    // sep "" ""
    UOps::ReadLine,
    UOps::JmpRelZ(7),
    UOps::Add,
    UOps::Add,
    // sep str
    UOps::Swap(1),
    UOps::Dup,
    // str sep sep
    UOps::Swap(2),
    UOps::Swap(1),
    // sep str sep
    UOps::JmpRel(-9),
    UOps::Pop,
    UOps::Swap(1),
    UOps::Pop,
    // str
    UOps::Print,
];

const KNOWN_OPS: [&str; 4] = ["+", "add", "*", "mul"];

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("expected any token of type {expected:?}, got token \"{tok:?}\" as location {range:?}")]
    UnexpectedToken {
        tok: TokenOwned,
        expected: Vec<TokenName>,
        range: LocRange,
    },

    #[error(
        "unknown operations \"{op}\" as location {range:?}, known operations are {KNOWN_OPS:?}"
    )]
    UnknownOperation { op: String, range: LocRange },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = core::result::Result<T, Error>;

pub struct Parser<'a> {
    tokenizer: &'a mut Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: &'a mut Tokenizer<'a>) -> Self {
        Self { tokenizer }
    }

    pub fn compile(&'a mut self) -> Result<UCode<'a>> {
        let uops: &[UOps] = match self.tokenizer.next_token()? {
            (Token::Ident(op), range) => match op {
                "+" | "add" => &ADD_UCODE,
                "*" | "mul" => &MULL_UCODE,
                "join" => &JOIN_UCODE,
                _ => {
                    return Err(Error::UnknownOperation {
                        op: op.to_string(),
                        range,
                    });
                }
            },
            (tok, range) => {
                return Err(Error::UnexpectedToken {
                    tok: tok.to_owned(),
                    expected: [TokenName::Eof].to_vec(),
                    range,
                });
            }
        };

        Ok(UCode {
            uops: uops.into(),
            tokenizer: self.tokenizer,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Num(u64),
    String(String),
}

pub enum ValueType {
    Num,
    String,
}

#[derive(Debug, Clone)]
pub enum UOps {
    Push(Value),
    Pop,
    JmpRelZ(isize),
    JmpRel(isize),
    Swap(usize),
    ReadString,
    ReadNum,
    ReadLine,
    Dup,
    Trim,
    Add,
    Mul,
    Eq,
    Print,
}

pub struct UCode<'a> {
    pub uops: Vec<UOps>,
    pub tokenizer: &'a mut Tokenizer<'a>,
}
