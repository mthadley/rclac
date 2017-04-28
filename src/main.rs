extern crate ansi_term;
extern crate rustyline;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use ansi_term::Color::{Green, Yellow};
use regex::Regex;
use rustyline::completion::Completer;
use rustyline::{Config, Editor};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::vec::Drain;

static PROMPT: &str = ">> ";

fn main() {
    let mut state = State::new();

    let config = Config::builder().tab_completion(false).build();
    let mut editor = Editor::<State>::with_config(config);

    loop {
        editor.set_completer(Some(state.to_owned()));

        match editor.readline(PROMPT) {
            Ok(line) => {
                state.eval(&line);
            }
            Err(_) => break,
        }

        println!("= {}",
                 Green.paint(format!("{}", state.peek().unwrap_or(&0))));
    }
}

#[derive(Clone)]
struct State {
    stack: Vec<isize>,
    vars: HashMap<String, isize>,
}

impl State {
    pub fn new() -> State {
        State {
            stack: Vec::with_capacity(0xFF),
            vars: HashMap::new(),
        }
    }

    pub fn exec(&mut self, op: Op) {
        match op {
            Op::Add => {
                self.pop2().map(|(a, b)| self.push(a + b));
            }
            Op::Clear => {
                self.clear();
            }
            Op::Double => {
                self.pop().map(|a| self.push(a * 2));
            }
            Op::Exp => {
                self.pop2().map(|(a, b)| self.push(b.pow(a as u32)));
            }
            Op::Fact => {
                self.pop().map(|a| self.push((1..a + 1).product()));
            }
            Op::Square => {
                self.pop().map(|a| self.push(a.pow(2)));
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
            Op::Prod => {
                let product = self.drain().product();
                self.push(product);
            }
            Op::Push(value) => {
                self.push(value);
            }
            Op::Sum => {
                let sum = self.drain().sum();
                self.push(sum);
            }
            Op::Swap => {
                self.pop2().map(|(a, b)| self.push(a).push(b));
            }
            Op::VarInit(name) => {
                self.pop().map(|a| self.add_var(name, a));
            }
            Op::VarRef(name) => {
                self.get_var(&name).map(|a| self.push(a));
            }
            Op::Noop => {}
        }
    }

    pub fn eval(&mut self, cmds: &str) -> &mut Self {
        for token in cmds.split_whitespace() {
            self.exec(token.into())
        }
        self
    }

    pub fn peek(&self) -> Option<&isize> {
        self.stack.last()
    }

    fn clear(&mut self) {
        self.stack.clear();
    }

    fn drain(&mut self) -> Drain<isize> {
        self.stack.drain(..)
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

    fn pop2(&mut self) -> Option<(isize, isize)> {
        if self.stack.len() > 1 {
            Some((self.pop().unwrap(), self.pop().unwrap()))
        } else {
            None
        }
    }

    fn add_var(&mut self, key: String, value: isize) {
        self.vars.insert(key, value);
    }

    fn get_var(&self, key: &String) -> Option<isize> {
        self.vars.get(key).cloned()
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
        let state_display = Yellow.paint(format!("{}", self.to_owned().eval(line)));
        Ok((0, vec![format!("{}", state_display)]))
    }
}

enum Op {
    Add,
    Clear,
    Double,
    Exp,
    Fact,
    Inv,
    Mul,
    Noop,
    Prod,
    Push(isize),
    Square,
    Sub,
    Sum,
    Swap,
    VarInit(String),
    VarRef(String),
}

impl<'a> From<&'a str> for Op {
    fn from(string: &str) -> Self {
        match string {
            "*" => Op::Mul,
            "**" => Op::Double,
            "+" => Op::Add,
            "-" => Op::Sub,
            "!" => Op::Fact,
            "^" => Op::Exp,
            "^^" => Op::Square,
            "c" => Op::Clear,
            "inv" => Op::Inv,
            "swap" => Op::Swap,
            "sum" => Op::Sum,
            "prod" => Op::Prod,
            token => parse_op(token),
        }
    }
}

fn parse_op(token: &str) -> Op {
    parse_var_init(token)
        .or_else(|| parse_var_ref(token))
        .or_else(|| parse_push(token))
        .unwrap_or(Op::Noop)
}

fn parse_push(token: &str) -> Option<Op> {
    isize::from_str(token).ok().map(Op::Push)
}

fn parse_var_init(token: &str) -> Option<Op> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"=([a-zA-Z][a-zA-Z0-9]*)").unwrap();
    }

    RE.captures(token)
        .and_then(|captures| captures.get(1))
        .map(|re_match| Op::VarInit(re_match.as_str().to_string()))
}

fn parse_var_ref(token: &str) -> Option<Op> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\$([a-zA-Z][a-zA-Z0-9]*)").unwrap();
    }

    RE.captures(token)
        .and_then(|captures| captures.get(1))
        .map(|re_match| Op::VarRef(re_match.as_str().to_string()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn exec() {
        let mut state = State::new();

        state.exec(Op::Push(3));
        state.exec(Op::Push(5));
        state.exec(Op::Add);
        assert_eq!(state.peek(), Some(&8));
    }

    #[test]
    fn eval() {
        let mut state = State::new();

        state.eval("3 5 +");
        assert_eq!(state.peek(), Some(&8));
    }

    #[test]
    fn variables() {
        let mut state = State::new();

        state.eval("3 =foo");
        assert_eq!(state.peek(), None);
        state.eval("$foo");
        assert_eq!(state.peek(), Some(&3));
    }
}
