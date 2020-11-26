use super::{Parser};
use crate::{token};
use crate::ast::expr::*;
use crate::ast::lit::*;
use crate::ast::ident::{Identifier};
use crate::ast::operator::{Prefix, Infix, BinaryOperator};

impl token::Token {
  fn to_binary_operator(&self) -> BinaryOperator {
    match self {
      token::Token::EQ | token::Token::NotEq => BinaryOperator::Equals,
      token::Token::LT | token::Token::GT => BinaryOperator::LtGt,
      token::Token::PLUS | token::Token::MINUS => BinaryOperator::Sum,
      token::Token::ASTERISK | token::Token::SLASH => BinaryOperator::Product,
      _ => BinaryOperator::Lowest,
    }
  }
}

impl Parser {
  pub(super) fn parse_expression(&mut self, op: BinaryOperator) -> Option<Expression> {
    let mut left = match self.parse_prefix() {
      Some(expr) => expr,
      None => return None,
    };
    
    while !self.peek_token.is(token::Token::SEMICOLON) && op < self.peek_token.to_binary_operator() {
      self.next_token();
      left = match self.parse_infix(left) {
        Some(expr) => expr,
        None => return None,
      };
    }

    Some(left)
  }

  fn parse_prefix(&mut self) -> Option<Expression> {
    match &self.current_token {
      token::Token::IDENT(s) => self.parse_identifier(s.to_string()),
      token::Token::INT(int) => self.parse_integer_literal(*int),
      token::Token::TRUE | token::Token::FALSE => self.parse_boolean_literal(),
      token::Token::BANG | token::Token::MINUS => self.parse_prefix_expression(),
      token::Token::LPAREN => self.parse_grouped_expression(),
      token::Token::IF => self.parse_if_expression(),
      _ => {
        self.no_prefix_parse_error();
        return None;
      }
    }
  }

  fn parse_infix(&mut self, left: Expression) -> Option<Expression> {
    let operator = match self.current_token {
      token::Token::PLUS => Infix::Plus,
      token::Token::MINUS => Infix::Minus,
      token::Token::SLASH => Infix::Slash,
      token::Token::ASTERISK => Infix::Asterisk,
      token::Token::GT => Infix::Gt,
      token::Token::LT => Infix::Lt,
      token::Token::EQ => Infix::Equal,
      token::Token::NotEq => Infix::NotEq,
      _ => return None,
    };

    let precedence = self.current_token.to_binary_operator();

    self.next_token();

    let right = match self.parse_expression(precedence) {
      Some(expr) => expr,
      None => return None,
    };

    let expr = Expression::Infix(InfixExpression::new(Box::new(left), operator, Box::new(right)));
    Some(expr)
  }

  fn parse_identifier(&self, value: String) -> Option<Expression> {
    Some(Expression::Identifier(Identifier::new(value)))
  }

  fn parse_integer_literal(&self, int: i64) -> Option<Expression> {
    Some(
      Expression::Literal(
        Literal::Integer(
          Integer::new(int),
        ),
      )
    )
  }

  fn parse_boolean_literal(&self) -> Option<Expression> {
    Some(
      Expression::Literal(
        Literal::Boolean(
          Boolean::new(self.current_token == token::Token::TRUE),
        ),
      ),
    )
  }

  fn parse_prefix_expression(&mut self) -> Option<Expression> {
    let operator = match self.current_token {
      token::Token::MINUS => Prefix::Minus,
      token::Token::BANG => Prefix::Bang,
      _ => {
        self.no_prefix_parse_error();
        return None;
      }
    };
    
    self.next_token();

    let right = match self.parse_expression(BinaryOperator::Prefix) {
      Some(expr) => expr,
      None => return None,
    };

    Some(Expression::Prefix(PrefixExpression::new(operator, Box::new(right))))
  }

  fn parse_grouped_expression(&mut self) -> Option<Expression> {
    self.next_token();

    let left = self.parse_expression(BinaryOperator::Lowest);

    if !self.expect_peek(token::Token::RPAREN) {
      return None;
    }

    left
  }

  fn parse_if_expression(&mut self) -> Option<Expression> {
    if !self.expect_peek(token::Token::LPAREN) {
      return None;
    }

    self.next_token();

    let condition = match self.parse_expression(BinaryOperator::Lowest) {
      Some(expr) => expr,
      None => return None,
    };

    if !self.expect_peek(token::Token::RPAREN) {
      return None;
    }

    if !self.expect_peek(token::Token::LBRACE) {
      return None;
    }

    let consequence = self.parse_block_statement();

    // Because ELSE token don't want to be error,
    // self.expect_peek() is not used
    let alternative = if self.peek_token.is(token::Token::ELSE) {
      self.next_token();
      if !self.expect_peek(token::Token::LBRACE) {
        return None;
      }
      Some(self.parse_block_statement())
    } else {
      None
    };

    Some(
      Expression::If(
        IfExpression::new(Box::new(condition), consequence, alternative),
      ),
    )
  }

  fn no_prefix_parse_error(&mut self) {
    let msg = format!("no prefix parse function for {:?}", self.current_token);
    self.errors.push(msg);
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::stmt::{Statement};
  use crate::lexer;
  use super::*;

  #[test]
  fn test_parse_identifier_expression() {
    let input = "foobar;";

    let l = lexer::Lexer::new(input.to_string());
    let mut p = Parser::new(l);

    let program = p.parse_program();
    if let Err(e) = p.check_parse_errors() {
      panic!("{}", e);
    }

    if program.statements.len() != 1 {
      panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
    }

    let expr = match &program.statements[0] {
      Statement::Expr(expr) => expr,
      _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[0]),
    };

    test_identifier(&expr.value, "foobar");
  }

  #[test]
  fn test_parse_int_literal_expression() {
    let input = "5;";

    let l = lexer::Lexer::new(input.to_string());
    let mut p = Parser::new(l);

    let program = p.parse_program();
    if let Err(e) = p.check_parse_errors() {
      panic!("{}", e);
    }

    if program.statements.len() != 1 {
      panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
    }

    let expr = match &program.statements[0] {
      Statement::Expr(expr) => expr,
      _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[0]),
    };

    test_integer_literal(&expr.value, &5);
  }
  
  #[test]
  fn test_boolean_expression() {
    let input = "
true;
false;
";

    let tests = vec![true, false];

    let l = lexer::Lexer::new(input.to_string());
    let mut p = Parser::new(l);

    let program = p.parse_program();
    if let Err(e) = p.check_parse_errors() {
      panic!("{}", e);
    }

    if program.statements.len() != 2 {
      panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
    }

    for (i, tt) in tests.iter().enumerate() {
      let expr = match &program.statements[i] {
        Statement::Expr(expr) => expr,
        _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[i]),
      };
  
      test_boolean(&expr.value, tt);
    }
  }
  
  #[test]
  fn test_parse_prefix_expression() {
    struct PrefixExpressionTest {
      input: String,
      operator: Prefix,
      right: ExpressionLiteral,
    }
    let prefix_expression_tests = vec![
      PrefixExpressionTest { 
        input: "!5;".to_string(),
        operator: Prefix::Bang,
        right: ExpressionLiteral::Int(5),
      },
      PrefixExpressionTest {
        input: "-15;".to_string(),
        operator: Prefix::Minus,
        right: ExpressionLiteral::Int(15),
      },
      PrefixExpressionTest {
        input: "!true;".to_string(),
        operator: Prefix::Bang,
        right: ExpressionLiteral::Bool(true),
      },
      PrefixExpressionTest {
        input: "!false;".to_string(),
        operator: Prefix::Bang,
        right: ExpressionLiteral::Bool(false),
      },
    ];

    for tt in prefix_expression_tests.into_iter() {
      let l = lexer::Lexer::new(tt.input.clone());
      let mut p = Parser::new(l);
  
      let program = p.parse_program();
      if let Err(e) = p.check_parse_errors() {
        panic!("{}", e);
      }
  
      if program.statements.len() != 1 {
        panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
      }
  
      let expr = match &program.statements[0] {
        Statement::Expr(expr) => expr,
        _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[0]),
      };
  
      let pre = match &expr.value {
        Expression::Prefix(pre) => pre,
        _ => panic!("Expression should has PrefixExpression, got {:?}", &expr.value)
      };

      if &pre.operator != &tt.operator {
        panic!("Prefix should has '{:?}', but got '{:?}'", &tt.operator, &pre.operator);
      }

      test_literal_expression(&pre.right, tt.right);
    }
  }

  #[test]
  fn test_parse_infix_expression() {
    struct InfixExpressionTest {
      input: String,
      left: ExpressionLiteral,
      operator: Infix,
      right: ExpressionLiteral,
    }
    let infix_expression_tests = vec![
      InfixExpressionTest {
        input: "5 + 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Plus,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 - 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Minus,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 * 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Asterisk,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 / 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Slash,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 > 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Gt,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 < 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Lt,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 == 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::Equal,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "5 != 5;".to_string(),
        left: ExpressionLiteral::Int(5),
        operator: Infix::NotEq,
        right: ExpressionLiteral::Int(5),
      },
      InfixExpressionTest { 
        input: "true == true;".to_string(),
        left: ExpressionLiteral::Bool(true),
        operator: Infix::Equal,
        right: ExpressionLiteral::Bool(true),
      },
      InfixExpressionTest { 
        input: "true != false;".to_string(),
        left: ExpressionLiteral::Bool(true),
        operator: Infix::NotEq,
        right: ExpressionLiteral::Bool(false),
      },
      InfixExpressionTest { 
        input: "false == false;".to_string(),
        left: ExpressionLiteral::Bool(false),
        operator: Infix::Equal,
        right: ExpressionLiteral::Bool(false),
      },
    ];

    for tt in infix_expression_tests.into_iter() {
      let l = lexer::Lexer::new(tt.input.clone());
      let mut p = Parser::new(l);
  
      let program = p.parse_program();
      if let Err(e) = p.check_parse_errors() {
        panic!("{}", e);
      }
  
      if program.statements.len() != 1 {
        panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
      }
  
      let expr = match &program.statements[0] {
        Statement::Expr(expr) => expr,
        _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[0]),
      };

      test_infix_expression(&expr.value, tt.left, tt.operator, tt.right);
    }
  }

  fn test_infix_expression(expr: &Expression, left: ExpressionLiteral, operator: Infix, right: ExpressionLiteral) {
    let inf = match expr {
      Expression::Infix(inf) => inf,
      _ => panic!("Expression should has InfixExpression, got {:?}", expr),
    };
    
    test_literal_expression(&inf.left, left);
    
    if &inf.operator != &operator {
      panic!("Infix should has '{:?}', but got '{:?}'", &operator, &inf.operator);
    }

    test_literal_expression(&inf.right, right);
  }

  #[test]
  fn test_operator_precedence_parsing() {
    struct PrecedenceTest {
      input: String,
      expected: String,
    }
    let precedence_tests = [
      PrecedenceTest { 
        input: "true".to_string(),
        expected: "true".to_string(),
      },
      PrecedenceTest { 
        input: "false".to_string(),
        expected: "false".to_string(),
      },
      PrecedenceTest { 
        input: "3 > 5 == false".to_string(),
        expected: "((3 > 5) == false)".to_string(),
      },
      PrecedenceTest { 
        input: "3 < 5 == true".to_string(),
        expected: "((3 < 5) == true)".to_string(),
      },
      PrecedenceTest { 
        input: "-a + b".to_string(),
        expected: "((-a) + b)".to_string(),
      },
      PrecedenceTest { 
        input: "!-a".to_string(),
        expected: "(!(-a))".to_string(),
      },
      PrecedenceTest { 
        input: "a + b + c".to_string(),
        expected: "((a + b) + c)".to_string(),
      },
      PrecedenceTest { 
        input: "a + b - c".to_string(),
        expected: "((a + b) - c)".to_string(),
      },
      PrecedenceTest { 
        input: "a * b * c".to_string(),
        expected: "((a * b) * c)".to_string(),
      },
      PrecedenceTest { 
        input: "a * b / c".to_string(),
        expected: "((a * b) / c)".to_string(),
      },
      PrecedenceTest { 
        input: "a + b / c".to_string(),
        expected: "(a + (b / c))".to_string(),
      },
      PrecedenceTest { 
        input: "a + b * c + d / e - f".to_string(),
        expected: "(((a + (b * c)) + (d / e)) - f)".to_string(),
      },
      PrecedenceTest { 
        input: "3 + 4; -5 * 5".to_string(),
        expected: "(3 + 4)((-5) * 5)".to_string(),
      },
      PrecedenceTest { 
        input: "5 > 4 == 3 < 4".to_string(),
        expected: "((5 > 4) == (3 < 4))".to_string(),
      },
      PrecedenceTest { 
        input: "5 < 4 != 3 > 4".to_string(),
        expected: "((5 < 4) != (3 > 4))".to_string(),
      },
      PrecedenceTest { 
        input: "3 + 4 * 5 == 3 * 1 + 4 * 5".to_string(),
        expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".to_string(),
      },
      PrecedenceTest { 
        input: "1 + (2 + 3) + 4".to_string(),
        expected: "((1 + (2 + 3)) + 4)".to_string(),
      },
      PrecedenceTest { 
        input: "(5 + 5) * 2".to_string(),
        expected: "((5 + 5) * 2)".to_string(),
      },
      PrecedenceTest { 
        input: "2 / (5 + 5)".to_string(),
        expected: "(2 / (5 + 5))".to_string(),
      },
      PrecedenceTest { 
        input: "-(5 + 5)".to_string(),
        expected: "(-(5 + 5))".to_string(),
      },
      PrecedenceTest { 
        input: "!(true == true)".to_string(),
        expected: "(!(true == true))".to_string(),
      },
    ];

    for tt in precedence_tests.iter() {
      let l = lexer::Lexer::new(tt.input.clone());
      let mut p = Parser::new(l);
  
      let program = p.parse_program();
      if let Err(e) = p.check_parse_errors() {
        panic!("{}", e);
      }
  
      let actual = format!("{}", program);
      if actual != tt.expected {
        panic!("expected={}, actual={}", tt.expected, actual);
      }
    }
  }

  #[test]
  fn test_parse_if_expression() {
    let input = "if (x < y) { x };";

    let l = lexer::Lexer::new(input.to_string());
    let mut p = Parser::new(l);

    let program = p.parse_program();
    if let Err(e) = p.check_parse_errors() {
      panic!("{}", e);
    }

    if program.statements.len() != 1 {
      panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
    }

    let expr = match &program.statements[0] {
      Statement::Expr(expr) => expr,
      _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[0]),
    };

    let if_expr = match &expr.value {
      Expression::If(if_expr) => if_expr,
      _ => panic!("Expression should has IfExpression, but got {:?}", expr.value)
    };

    test_infix_expression(
      &if_expr.condition,
      ExpressionLiteral::Str("x".to_string()),
      Infix::Lt,
      ExpressionLiteral::Str("y".to_string()),
    );

    if if_expr.consequence.statements.len() != 1 {
      panic!(
        "if_expr.consequence.statements should has only 1 statement, but got {}",
        if_expr.consequence.statements.len(),
      );
    }

    let con_expr = match &if_expr.consequence.statements[0] {
      Statement::Expr(expr) => expr,
      _ => panic!(
        "if_expr.consequence.statements[0] should has ExpressionStatement, but got {:?}",
        if_expr.consequence.statements[0],
      ),
    };

    test_identifier(&con_expr.value, "x");

    if let Some(_) = if_expr.alternative {
      panic!("if_expr.alternative should be None, but {:?}", if_expr.alternative);
    }
  }

  #[test]
  fn test_parse_else_expression() {
    let input = "if (x < y) { x } else { y };";

    let l = lexer::Lexer::new(input.to_string());
    let mut p = Parser::new(l);

    let program = p.parse_program();
    if let Err(e) = p.check_parse_errors() {
      panic!("{}", e);
    }

    if program.statements.len() != 1 {
      panic!("program.statements should has only 1 statement, but got {}", program.statements.len());
    }

    let expr = match &program.statements[0] {
      Statement::Expr(expr) => expr,
      _ => panic!("program.statements should has ExpressionStatement, but got {:?}", program.statements[0]),
    };

    let if_expr = match &expr.value {
      Expression::If(if_expr) => if_expr,
      _ => panic!("Expression should has IfExpression, but got {:?}", expr.value)
    };

    test_infix_expression(
      &if_expr.condition,
      ExpressionLiteral::Str("x".to_string()),
      Infix::Lt,
      ExpressionLiteral::Str("y".to_string()),
    );

    if if_expr.consequence.statements.len() != 1 {
      panic!(
        "if_expr.consequence.statements should has only 1 statement, but got {}",
        if_expr.consequence.statements.len(),
      );
    }

    let con_expr = match &if_expr.consequence.statements[0] {
      Statement::Expr(con_expr) => con_expr,
      _ => panic!(
        "if_expr.consequence.statements[0] should has ExpressionStatement, but got {:?}",
        if_expr.consequence.statements[0],
      ),
    };

    test_identifier(&con_expr.value, "x");

    let else_expr = match &if_expr.alternative {
      Some(else_expr) => else_expr,
      None => panic!("if_expr.alternative should has alternative, but got {:?}", if_expr.alternative),
    };

    let alt_expr = match &else_expr.statements[0] {
      Statement::Expr(alt_expr) => alt_expr,
      _ => panic!("if_expr.alternative should has alternative, but got {:?}", if_expr.alternative),
    };

    test_identifier(&alt_expr.value, "y");
  }

  enum ExpressionLiteral {
    Int(i64),
    Bool(bool),
    Str(String),
  }

  fn test_literal_expression(expr: &Expression, expect: ExpressionLiteral) {
    match expect {
      ExpressionLiteral::Int(v) => test_integer_literal(expr, &v),
      ExpressionLiteral::Str(v) => test_identifier(expr, &v),
      ExpressionLiteral::Bool(v) => test_boolean(expr, &v),
    };
  }

  fn test_identifier(expr: &Expression, value: &str) {
    let ident = match &expr {
      Expression::Identifier(ident) => ident,
      _ => panic!("Expression should has Identifier, got {}", &expr)
    };

    if &ident.value != value {
      panic!("Identifier should has foobar, but got {}", &ident.value);
    }
  }

  fn test_integer_literal(expr: &Expression, comp: &i64) {
    let lit = match &expr {
      Expression::Literal(lit) => lit,
      _ => panic!("Expression should has Literal, got {}", &expr)
    };
    
    let int = match &lit {
      Literal::Integer(int) => int,
      _ => panic!("Literal should has Integer, got {}", &lit)
    };

    if &int.value != comp {
      panic!("Identifier should has {}, but got {}", int.value, comp);
    }
  }

  fn test_boolean(expr: &Expression, comp: &bool) {
    let lit = match &expr {
      Expression::Literal(lit) => lit,
      _ => panic!("Expression should has Literal, got {}", &expr)
    };
    
    let int = match &lit {
      Literal::Boolean(v) => v,
      _ => panic!("Literal should has Boolean, got {}", &lit)
    };

    if &int.value != comp {
      panic!("Identifier should has {}, but got {}", int.value, comp);
    }
  }
}