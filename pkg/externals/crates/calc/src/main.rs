use whatsrook_sdk::{respond, Request};

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("Usage: calc [expression] (e.g. calc 2 + 2 * 4, calc sqrt(144) + 2^3)");
        return;
    }

    match evaluate(&query) {
        Ok(result) => {
            let formatted = if result.is_nan() {
                "NaN".to_string()
            } else if result.is_infinite() {
                if result.is_sign_positive() {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else if result.fract() == 0.0 && result.abs() < 1e15 {
                format!("{}", result as i64)
            } else {
                format!("{:.8}", result)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            };
            respond(format!("Result: {}", formatted));
        }
        Err(err) => {
            respond(format!("Math error: {}", err));
        }
    }
}

pub fn evaluate(expr: &str) -> Result<f64, String> {
    let cleaned: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return Err("empty expression".to_string());
    }
    let mut parser = Parser::new(&cleaned);
    let val = parser.parse_expression()?;
    if parser.pos < parser.input.len() {
        return Err(format!(
            "unexpected token {:?} at position {}",
            &parser.input[parser.pos..],
            parser.pos
        ));
    }
    Ok(val)
}

struct Parser<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;

        while let Some(c) = self.peek() {
            if c == '+' || c == '-' {
                self.next_char();
                let right = self.parse_term()?;
                if c == '+' {
                    left += right;
                } else {
                    left -= right;
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_power()?;

        while let Some(c) = self.peek() {
            if c == '*' || c == '/' || c == '%' {
                self.next_char();
                let right = self.parse_power()?;
                match c {
                    '*' => left *= right,
                    '/' => {
                        if right == 0.0 {
                            return Err("division by zero".to_string());
                        }
                        left /= right;
                    }
                    '%' => {
                        if right == 0.0 {
                            return Err("modulo by zero".to_string());
                        }
                        left %= right;
                    }
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<f64, String> {
        let left = self.parse_factor()?;

        if let Some('^') = self.peek() {
            self.next_char();
            let right = self.parse_power()?; // Right-associative
            Ok(left.powf(right))
        } else {
            Ok(left)
        }
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some('+') => {
                self.next_char();
                self.parse_factor()
            }
            Some('-') => {
                self.next_char();
                let val = self.parse_factor()?;
                Ok(-val)
            }
            Some('(') => {
                self.next_char();
                let val = self.parse_expression()?;
                if self.peek() != Some(')') {
                    return Err("missing closing parenthesis".to_string());
                }
                self.next_char();
                Ok(val)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => self.parse_number(),
            Some(c) if c.is_alphabetic() => self.parse_identifier_or_function(),
            Some(c) => Err(format!("unexpected character {:?}", c)),
            None => Err("unexpected end of expression".to_string()),
        }
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.next_char();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.next_char();
            } else {
                break;
            }
        }

        let num_str: String = self.chars[start..self.pos].iter().collect();
        num_str
            .parse::<f64>()
            .map_err(|e| format!("invalid number {:?}: {}", num_str, e))
    }

    fn parse_identifier_or_function(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.next_char();
            } else {
                break;
            }
        }

        let ident: String = self.chars[start..self.pos].iter().collect();
        let lower = ident.to_lowercase();

        // Check constants
        match lower.as_str() {
            "pi" => return Ok(std::f64::consts::PI),
            "e" => return Ok(std::f64::consts::E),
            "tau" => return Ok(std::f64::consts::TAU),
            _ => {}
        }

        // Functions require parenthesis
        if self.peek() != Some('(') {
            return Err(format!("unknown identifier {:?}; expected '('", ident));
        }

        self.next_char(); // consume '('
        let arg = self.parse_expression()?;

        if self.peek() != Some(')') {
            return Err(format!(
                "missing closing parenthesis for function {:?}",
                ident
            ));
        }
        self.next_char(); // consume ')'

        match lower.as_str() {
            "sqrt" => {
                if arg < 0.0 {
                    Err("square root of negative number".to_string())
                } else {
                    Ok(arg.sqrt())
                }
            }
            "abs" => Ok(arg.abs()),
            "sin" => Ok(arg.sin()),
            "cos" => Ok(arg.cos()),
            "tan" => Ok(arg.tan()),
            "asin" => Ok(arg.asin()),
            "acos" => Ok(arg.acos()),
            "atan" => Ok(arg.atan()),
            "log" | "log10" => {
                if arg <= 0.0 {
                    Err("log of non-positive number".to_string())
                } else {
                    Ok(arg.log10())
                }
            }
            "ln" => {
                if arg <= 0.0 {
                    Err("ln of non-positive number".to_string())
                } else {
                    Ok(arg.ln())
                }
            }
            "exp" => Ok(arg.exp()),
            "floor" => Ok(arg.floor()),
            "ceil" => Ok(arg.ceil()),
            "round" => Ok(arg.round()),
            _ => Err(format!("unknown function {:?}", ident)),
        }
    }
}
