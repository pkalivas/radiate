use crate::{Op, Tree, TreeNode};

pub trait Expression<T> {
    fn parse(expr: &str) -> Result<T, String>;
}

impl Expression<Tree<Op<f32>>> for Tree<Op<f32>> {
    fn parse(expr: &str) -> Result<Tree<Op<f32>>, String> {
        parse(expr).map(|node| Tree::new(node))
    }
}

fn parse(expr: &str) -> Result<TreeNode<Op<f32>>, String> {
    let tokens = tokenize(expr);
    let mut pos = 0;
    parse_expression(&tokens, &mut pos)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f32),
    Identifier(String, Option<usize>),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LParen,
    RParen,
    EOF,
}

fn tokenize(expression: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = expression.chars().peekable();
    let mut vars = Vec::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        num.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(ident.clone(), None));
                vars.push(ident);
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Multiply);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Divide);
            }
            '^' => {
                chars.next();
                tokens.push(Token::Power);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            _ => panic!("Unexpected character: {}", ch),
        }
    }

    tokens.push(Token::EOF);
    vars.dedup();
    vars.sort();

    for i in 0..tokens.len() {
        if matches!(tokens[i], Token::Identifier(_, _)) {
            let name = match &tokens[i] {
                Token::Identifier(name, _) => name,
                _ => unreachable!(),
            };
            let index = vars.iter().position(|v| v == name).unwrap();
            tokens[i] = Token::Identifier(name.clone(), Some(index));
        }
    }

    tokens
}

fn parse_expression(tokens: &[Token], pos: &mut usize) -> Result<TreeNode<Op<f32>>, String> {
    let mut node = parse_term(tokens, pos)?;

    while let Some(token) = tokens.get(*pos) {
        match token {
            Token::Plus | Token::Minus => {
                let op = token.clone();
                *pos += 1;
                let right = parse_term(tokens, pos)?;
                node = TreeNode::new(match op {
                    Token::Plus => Op::add(),
                    Token::Minus => Op::sub(),
                    _ => unreachable!(),
                })
                .attach(node)
                .attach(right);
            }
            _ => break,
        }
    }

    Ok(node)
}

fn parse_term(tokens: &[Token], pos: &mut usize) -> Result<TreeNode<Op<f32>>, String> {
    let mut node = parse_power(tokens, pos)?;

    while let Some(token) = tokens.get(*pos) {
        match token {
            Token::Multiply | Token::Divide => {
                let op = token.clone();
                *pos += 1;
                let right = parse_power(tokens, pos)?;
                node = TreeNode::new(match op {
                    Token::Multiply => Op::mul(),
                    Token::Divide => Op::div(),
                    _ => unreachable!(),
                })
                .attach(node)
                .attach(right);
            }
            _ => break,
        }
    }

    Ok(node)
}

fn parse_power(tokens: &[Token], pos: &mut usize) -> Result<TreeNode<Op<f32>>, String> {
    let mut node = parse_factor(tokens, pos)?;

    if let Some(Token::Power) = tokens.get(*pos) {
        *pos += 1;
        let right = parse_power(tokens, pos)?;
        node = TreeNode::new(Op::pow()).attach(node).attach(right);
    }

    Ok(node)
}

fn parse_factor(tokens: &[Token], pos: &mut usize) -> Result<TreeNode<Op<f32>>, String> {
    match tokens.get(*pos) {
        Some(Token::Minus) => {
            *pos += 1;
            Ok(TreeNode::new(Op::neg()).attach(parse_factor(tokens, pos)?))
        }
        Some(Token::Plus) => {
            *pos += 1;
            parse_factor(tokens, pos)
        }
        Some(Token::Number(n)) => {
            *pos += 1;
            Ok(TreeNode::new(Op::constant(*n)))
        }
        Some(Token::Identifier(_, var)) => {
            *pos += 1;
            Ok(TreeNode::new(Op::var(var.unwrap())))
        }
        Some(Token::LParen) => {
            *pos += 1;
            let node = parse_expression(tokens, pos)?;
            if let Some(Token::RParen) = tokens.get(*pos) {
                *pos += 1;
                Ok(node)
            } else {
                Err("Expected ')'".to_string())
            }
        }
        token => Err(format!("Unexpected token: {:?}", token)),
    }
}

#[cfg(test)]
mod test {
    use crate::{Eval, Tree, ops::expr::Expression};

    #[test]
    fn test_tokenize() {
        let expr_str = "1 + 2 * (3 * 4)^5";
        if let Ok(tree) = Tree::parse(expr_str) {
            assert_eq!(tree.eval(&[]), 497665.0);
        } else {
            panic!("Failed to parse expression");
        }
    }

    #[test]
    fn test_tokenize_with_vars() {
        let expr_str = "a + b * (c * d)^e";

        if let Ok(tree) = Tree::parse(expr_str) {
            assert_eq!(tree.eval(&[1.0, 2.0, 3.0, 4.0, 5.0]), 497665.0);
        } else {
            panic!("Failed to parse expression");
        }
    }

    #[test]
    fn test_tokenize_with_vars_and_negation() {
        let expr_str = "5 - x * (34 * 3)^2";

        if let Ok(tree) = Tree::parse(expr_str) {
            assert_eq!(tree.eval(&[3.0]), -31207.0);
        } else {
            panic!("Failed to parse expression");
        }
    }

    #[test]
    fn test_tokenize_with_vars_and_negation_and_parens() {
        let comp = |x: f32| 4.0 * x.powf(3.0) - 3.0 * x.powf(2.0) + x;

        let expr_str = "4 * x^3 - 3 * x^2 + x";

        if let Ok(tree) = Tree::parse(expr_str) {
            let mut input = -1.0;
            for _ in -10..10 {
                input += 0.1;
                let output = tree.eval(&[input]);
                assert!((output - comp(input)).abs() < 0.0001);
            }
        } else {
            panic!("Failed to parse expression");
        }
    }
}
