use crate::{
    parser::{UCode, UOps, Value},
    tokenizer::Token,
};

#[derive(Debug)]
pub struct Regs {
    pub ip: usize,
}

pub fn run(ucode: UCode) {
    let mut stack = vec![];
    let mut regs = Regs { ip: 0 };

    let pop_2 = |stack: &mut Vec<_>| {
        if stack.len() < 2 {
            return None;
        }

        Some([stack.pop()?, stack.pop()?])
    };

    loop {
        let Some(i) = ucode.uops.get(regs.ip) else {
            // Should that be ok?
            return;
        };

        log::debug!("{:?} \t {:?} {:?}", regs, i, stack);

        match i {
            UOps::Push(value) => stack.push(value.clone()),
            UOps::JmpRelZ(offset) => {
                if let Some(Value::Num(n)) = stack.pop() {
                    if n == 0 {
                        regs.ip = regs.ip.saturating_add_signed(*offset);
                    }
                } else {
                    todo!()
                }
            }
            UOps::JmpRel(offset) => regs.ip = regs.ip.saturating_add_signed(*offset),
            UOps::ReadNum => match ucode.tokenizer.read_num().unwrap().0 {
                Token::Value(v @ Value::Num(_)) => {
                    stack.push(v);
                    stack.push(Value::Num(1))
                }
                _ => stack.push(Value::Num(0)),
            },
            UOps::ReadString => match ucode.tokenizer.read_string().unwrap().0 {
                Token::Value(v @ Value::String(_)) => {
                    stack.push(v);
                    stack.push(Value::Num(1))
                }
                _ => stack.push(Value::Num(0)),
            },
            UOps::ReadLine => match ucode.tokenizer.read_line().unwrap().0 {
                Token::Value(v @ Value::String(_)) => {
                    stack.push(v);
                    stack.push(Value::Num(1))
                }
                _ => stack.push(Value::Num(0)),
            },
            UOps::Trim => {
                if let Some(Value::String(s)) = stack.pop() {
                    stack.push(Value::String(s.trim().to_string()));
                } else {
                    todo!()
                }
            }
            UOps::Add => {
                if let Some(s) = pop_2(&mut stack) {
                    match s {
                        [Value::Num(v2), Value::Num(v1)] => stack.push(Value::Num(v1 + v2)),
                        [Value::String(v2), Value::String(v1)] => {
                            stack.push(Value::String(v1 + &v2))
                        }
                        _ => todo!(),
                    }
                } else {
                    todo!()
                }
            }
            UOps::Eq => {
                if let Some([v1, v2]) = pop_2(&mut stack) {
                    stack.push(Value::Num(if v1 == v2 { 1 } else { 0 }))
                } else {
                    todo!()
                }
            }
            UOps::Dup => {
                if let Some(v) = stack.pop() {
                    stack.push(v.clone());
                    stack.push(v);
                } else {
                    todo!()
                }
            }
            UOps::Print => {
                if let Some(v) = stack.pop() {
                    match v {
                        Value::Num(n) => println!("{}", n),
                        Value::String(s) => println!("{}", s),
                    }
                } else {
                    todo!()
                }
            }
            UOps::Swap(n) => {
                if *n != 0 {
                    if stack.len() <= *n {
                        todo!()
                    }

                    if let Some(mut x) = stack.pop() {
                        let l = stack.len();
                        std::mem::swap(&mut stack[l - n], &mut x);
                        stack.push(x);
                    }
                }
            }
            UOps::Pop => {
                if stack.pop().is_some() {
                } else {
                    todo!()
                }
            }
        }
        regs.ip += 1;
    }
}
