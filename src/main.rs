extern crate ansi_term;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rustyline;

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

        println!(
            "= {}",
            Green.paint(format!("{}", state.peek().unwrap_or(&0)))
        );
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
                self.apply2(|a, b| a + b);
            }
            Op::Clear => {
                self.clear();
            }
            Op::Div => {
                self.apply2(|a, b| save_div(b, a).unwrap_or(0));
            }
            Op::Double => {
                self.apply(|a| a * 2);
            }
            Op::Exp => {
                self.apply2(|a, b| b.pow(a as u32));
            }
            Op::Fact => {
                self.apply(|a| (1..a + 1).product());
            }
            Op::Square => {
                self.apply(|a| a.pow(2));
            }
            Op::Sub => {
                self.apply2(|a, b| b - a);
            }
            Op::Mul => {
                self.apply2(|a, b| a * b);
            }
            Op::Inv => {
                self.apply(|a| -a);
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
                if let Some((a, b)) = self.pop2() {
                    self.push(a).push(b);
                }
            }
            Op::VarInit(name) => {
                if let Some(a) = self.stack.pop() {
                    self.add_var(name, a);
                }
            }
            Op::VarRef(name) => {
                if let Some(a) = self.get_var(&name) {
                    self.push(a);
                }
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

    fn apply(&mut self, func: impl FnOnce(isize) -> isize) {
        if let Some(val) = self.stack.pop().map(func) {
            self.push(val);
        }
    }

    fn apply2(&mut self, func: impl FnOnce(isize, isize) -> isize) {
        if let Some((a, b)) = self.pop2() {
            self.stack.push(func(a, b));
        }
    }

    fn pop2(&mut self) -> Option<(isize, isize)> {
        if self.stack.len() > 1 {
            Some((self.stack.pop().unwrap(), self.stack.pop().unwrap()))
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
    Div,
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
            "/" => Op::Div,
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

lazy_static! {
    static ref INIT_RE: Regex = Regex::new(r"=([a-zA-Z][a-zA-Z0-9]*)").unwrap();
}

fn parse_var_init(token: &str) -> Option<Op> {
    INIT_RE
        .captures(token)
        .and_then(|captures| captures.get(1))
        .map(|re_match| Op::VarInit(re_match.as_str().to_string()))
}

lazy_static! {
    static ref VAR_RE: Regex = Regex::new(r"\$([a-zA-Z][a-zA-Z0-9]*)").unwrap();
}

fn parse_var_ref(token: &str) -> Option<Op> {
    VAR_RE
        .captures(token)
        .and_then(|captures| captures.get(1))
        .map(|re_match| Op::VarRef(re_match.as_str().to_string()))
}

fn save_div(a: isize, b: isize) -> Option<isize> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
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

    #[test]
    fn test_save_div() {
        assert_eq!(save_div(24, 2), Some(12));
        assert_eq!(save_div(24, 0), None);
    }
}
