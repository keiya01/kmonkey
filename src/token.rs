use std::cmp::PartialEq;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
  ILLEGAL,
  EOF,
  
  // 識別子 + リテラル
  IDENT(&'a str),
  INT(i64),
  
  // 演算子
  ASSIGN,
  PLUS,
  MINUS,
  BANG,
  ASTERISK,
  SLASH,

  LT,
  GT,
  EQ,
  NotEq,
  
  // デリミタ
  COMMA,
  SEMICOLON,
  
  LPAREN,
  RPAREN,
  LBRACE,
  RBRACE,
  
  // キーワード
  FUNCTION,
  LET,
  TRUE,
  FALSE,
  IF,
  ELSE,
  RETURN,
}