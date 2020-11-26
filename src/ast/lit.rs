use std::fmt;
use super::ident::Identifier;
use super::stmt::BlockStatement;

#[derive(Debug)]
pub enum Literal {
  Integer(Integer),
  Boolean(Boolean),
  Func(Func),
}

impl fmt::Display for Literal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Literal::Integer(int) => write!(f, "{}", int),
      Literal::Boolean(v) => write!(f, "{}", v),
      Literal::Func(func) => write!(f, "{}", func),
    }
  }
}

#[derive(Debug)]
pub struct Integer {
  pub value: i64,
}

impl Integer {
  pub fn new(value: i64) -> Integer {
    Integer { value }
  }
}

impl fmt::Display for Integer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", &self.value)
  }
}

#[derive(Debug)]
pub struct Boolean {
  pub value: bool,
}

impl Boolean {
  pub fn new(value: bool) -> Boolean {
    Boolean { value }
  }
}

impl fmt::Display for Boolean {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", &self.value)
  }
}

#[derive(Debug)]
pub struct Func {
  pub args: Vec<Identifier>,
  pub body: BlockStatement,
}

impl Func {
  pub fn new(args: Vec<Identifier>, body: BlockStatement) -> Func {
    Func { args, body }
  }
}

impl fmt::Display for Func {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "fn(")?;
    for arg in &self.args {
      write!(f, "{}, ", &arg.value)?;
    }
    write!(f, ") {}", &self.body)?;
    Ok(())
  }
}
