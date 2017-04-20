extern crate ansi_term;
extern crate rustyline;

use ansi_term::Color::{Green, Blue};
use rustyline::completion::Completer;
use rustyline::Editor;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

static PROMPT: &'static str = ">> ";

fn main() {
    let mut state = State::new();
    let mut editor = Editor::<State>::new();

    loop {
        editor.set_completer(Some(state.to_owned()));

        match editor.readline(PROMPT) {
            Ok(line) => {
                state.eval(&line);
            }
            Err(_) => break,
        }

        println!("{}", Green.paint(format!("{}", state.peek().unwrap_or(&0))));
    }
}

#[derive(Clone)]
struct State {
    stack: Vec<isize>,
}

impl State {
    pub fn new() -> State {
        State { stack: Vec::with_capacity(0xFF) }
    }

    pub fn pop2(&mut self) -> Option<(isize, isize)> {
        if self.stack.len() > 1 {
            Some((self.pop().unwrap(), self.pop().unwrap()))
        } else {
            None
        }
    }

    pub fn exec(&mut self, op: &Op) {
        match *op {
            Op::Add => {
                self.pop2().map(|(a, b)| self.push(a + b));
            }
            Op::Sub => {
                self.pop2().map(|(a, b)| self.push(b - a));
            }
            Op::Mul => {
                self.pop2().map(|(a, b)| self.push(a * b));
            }
            Op::Inv => {
                self.pop().map(|a| self.push(-a));
            }
            Op::Push(value) => {
                self.push(value);
            }
            Op::Swap => {
                self.pop2().map(|(a, b)| {
                    self.push(a).push(b);
                });
            }
            Op::Print => println!("{}", Blue.paint(format!("{}", self))),
            Op::Noop => {}
        }
    }

    pub fn eval(&mut self, cmds: &str) -> &mut Self {
        for token in cmds.split_whitespace() {
            self.exec(&token.into())
        }
        self
    }

    pub fn peek(&self) -> Option<&isize> {
        self.stack.last()
    }

    fn push(&mut self, val: isize) -> &mut Self {
        self.stack.push(val);
        self
    }

    fn pop(&mut self) -> Option<isize> {
        if self.stack.len() > 0 {
            self.stack.pop()
        } else {
            None
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for val in self.stack.iter() {
            write!(f, "{} ", val)?
        }
        Ok(())
    }
}

impl Completer for State {
    fn complete(&self, line: &str, _: usize) -> rustyline::Result<(usize, Vec<String>)> {
        Ok((0, vec![format!("{}", self.to_owned().eval(line))]))
    }
}

enum Op {
    Add,
    Inv,
    Mul,
    Noop,
    Push(isize),
    Sub,
    Swap,
    Print,
}

impl<'a> From<&'a str> for Op {
    fn from(string: &'a str) -> Self {
        match string {
            "p" => Op::Print,
            "*" => Op::Mul,
            "+" => Op::Add,
            "-" => Op::Sub,
            "inv" => Op::Inv,
            "swap" => Op::Swap,
            string => {
                isize::from_str(string)
                    .map(|val| Op::Push(val))
                    .unwrap_or(Op::Noop)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn exec() {
        let mut state = State::new();

        state.exec(&Op::Push(3));
        state.exec(&Op::Push(5));
        state.exec(&Op::Add);
        assert_eq!(state.peek(), Some(&8));
    }

    #[test]
    fn eval() {
        let mut state = State::new();

        state.eval("3 5 +");
        assert_eq!(state.peek(), Some(&8));
    }
}
