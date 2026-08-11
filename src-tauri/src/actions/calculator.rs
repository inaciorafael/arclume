#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Sqrt,
}

pub fn evaluate(input: &str) -> Result<f64, String> {
    if !looks_like_expression(input) {
        return Err("not a calculation".into());
    }
    let tokens = tokenize(input)?;
    let mut parser = Parser {
        tokens: &tokens,
        position: 0,
    };
    let value = parser.expression(0)?;
    if parser.position != tokens.len() {
        return Err("unexpected token".into());
    }
    if !value.is_finite() {
        return Err("result is not finite".into());
    }
    Ok(value)
}

fn looks_like_expression(input: &str) -> bool {
    let value = input.trim();
    !value.is_empty()
        && (value.starts_with("sqrt")
            || (value.chars().any(|character| "+-*/%^".contains(character))
                && value.chars().any(|character| character.is_ascii_digit())))
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut characters = input.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if character.is_whitespace() {
            continue;
        }
        let token = match character {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '^' => Token::Caret,
            '(' => Token::LParen,
            ')' => Token::RParen,
            value if value.is_ascii_digit() || value == '.' => {
                let mut end = index + value.len_utf8();
                while let Some((next_index, next)) = characters.peek().copied() {
                    if !next.is_ascii_digit() && next != '.' {
                        break;
                    }
                    characters.next();
                    end = next_index + next.len_utf8();
                }
                Token::Number(input[index..end].parse().map_err(|_| "invalid number")?)
            }
            value if value.is_alphabetic() => {
                let mut word = value.to_string();
                while let Some((_, next)) = characters.peek().copied() {
                    if !next.is_alphabetic() {
                        break;
                    }
                    characters.next();
                    word.push(next);
                }
                if word.eq_ignore_ascii_case("sqrt") {
                    Token::Sqrt
                } else {
                    return Err("unknown function".into());
                }
            }
            _ => return Err("invalid character".into()),
        };
        tokens.push(token);
    }
    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl Parser<'_> {
    fn expression(&mut self, minimum_binding_power: u8) -> Result<f64, String> {
        let mut left = match self.next().ok_or("expected value")? {
            Token::Number(value) => value,
            Token::Minus => -self.expression(6)?,
            Token::Plus => self.expression(6)?,
            Token::Sqrt => {
                let value = self.expression(9)?;
                if value < 0.0 {
                    return Err("square root requires a non-negative value".into());
                }
                value.sqrt()
            }
            Token::LParen => {
                let value = self.expression(0)?;
                if self.next() != Some(Token::RParen) {
                    return Err("missing closing parenthesis".into());
                }
                value
            }
            _ => return Err("expected value".into()),
        };
        while let Some(operator) = self.peek() {
            let (left_bp, right_bp) = match operator {
                Token::Plus | Token::Minus => (1, 2),
                Token::Star | Token::Slash | Token::Percent => (3, 4),
                Token::Caret => (7, 6),
                _ => break,
            };
            if left_bp < minimum_binding_power {
                break;
            }
            let operator = self.next().unwrap();
            let right = self.expression(right_bp)?;
            left = match operator {
                Token::Plus => left + right,
                Token::Minus => left - right,
                Token::Star => left * right,
                Token::Slash if right == 0.0 => return Err("division by zero".into()),
                Token::Slash => left / right,
                Token::Percent if right == 0.0 => return Err("division by zero".into()),
                Token::Percent => left % right,
                Token::Caret => left.powf(right),
                _ => unreachable!(),
            };
        }
        Ok(left)
    }
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }
    fn next(&mut self) -> Option<Token> {
        let value = self.peek();
        self.position += usize::from(value.is_some());
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn respects_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
    }
    #[test]
    fn supports_parentheses_and_sqrt() {
        assert_eq!(evaluate("sqrt(9) + (2^3)").unwrap(), 11.0);
    }
    #[test]
    fn exponent_precedes_unary_minus() {
        assert_eq!(evaluate("-2^2").unwrap(), -4.0);
    }
    #[test]
    fn rejects_division_by_zero_and_code() {
        assert!(evaluate("1 / 0").is_err());
        assert!(evaluate("process.exit()").is_err());
    }
}
