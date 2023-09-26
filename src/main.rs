use std::env;

struct ParsingError {
    error: String,
    base: String,
    index: usize,
}

impl ParsingError {
    fn new(error: String, base: String, index: usize) -> Self {
        Self { error, base, index }
    }
}

impl std::fmt::Debug for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsingError")
            .field("error", &self.error)
            .field("base", &self.base)
            .field("index", &self.index)
            .finish()
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.error)?;
        writeln!(f, "{}", self.base)?;
        writeln!(f, "{}^", " ".repeat(self.index))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Pow,
    Inverse,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Pow => write!(f, "^"),
            Self::Inverse => write!(f, "-"),
        }
    }
}

impl std::str::FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Subtract),
            "*" => Ok(Self::Multiply),
            "/" => Ok(Self::Divide),
            "^" => Ok(Self::Pow),
            _ => Err(format!("Invalid operator: {}", s)),
        }
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl Ord for Operator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl Operator {
    fn count(&self) -> i32 {
        // Return the number of arguments this operator takes
        match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Pow => 2,
            Self::Inverse => 1,
        }
    }

    fn check_arguments(&self, arguments: &Vec<i32>) -> Result<(), &str> {
        // Check if the arguments are valid for this operator
        match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide => {
                if arguments.len() != 2 {
                    return Err("Invalid number of arguments");
                }
            }
            Self::Inverse => {
                if arguments.len() != 1 {
                    return Err("Invalid number of arguments");
                }
            }
            Self::Pow => {
                if arguments.len() != 2 {
                    return Err("Invalid number of arguments");
                }
            }
        }
        Ok(())
    }

    fn priority(&self) -> i32 {
        // Return the priority of this operator
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide => 2,
            Self::Pow => 3,
            Self::Inverse => 4,
        }
    }

}

enum ParsingToken {
    Number(f64),
    Operator(Operator),
    Parenthesis(Vec<ParsingToken>),
}

impl ParsingToken {
    fn build_number(int_part: &str, float_part: &str) -> Result<f64, String> {
        let mut number = String::new();
        number.push_str(int_part);
        number.push('.');
        number.push_str(float_part);
        match number.parse() {
            Ok(n) => Ok(n),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn tokenize(input: &str) -> Result<Vec<Self>, ParsingError> {
        let mut tokens = Vec::new();

        let mut is_float = false;
        let mut is_parsing_parentesis = false;

        let mut current_number = String::new();
        let mut current_float = String::new();

        let mut parenthesis_depth = 0;
        let mut parenthesis_start = 0;

        let compute_number = |int_part: &mut String,
                              float_part: &mut String,
                              tokens: &mut Vec<ParsingToken>,
                              is_float: &mut bool,
                              index: usize|
         -> Result<(), ParsingError> {
            // if previous token is a parenthesis,
            // we do not want to compute the number
            // return Ok(()) if is_parsing_parentesis;
            if tokens
                .last()
                .map_or(false, |t| matches!(t, Self::Parenthesis(_)))
            {
                return Ok(());
            }

            let number = Self::build_number(int_part, float_part)
                .map_err(|e| ParsingError::new(e, input.to_string(), index))?;

            tokens.push(Self::Number(number));
            *int_part = String::new();
            *float_part = String::new();
            *is_float = false;
            Ok(())
        };

        for (i, c) in input.chars().enumerate() {
            match c {
                '0'..='9' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    // if last token is a parenthesis,
                    // we add a multiplication operator
                    if tokens
                        .last()
                        .map_or(false, |t| matches!(t, Self::Parenthesis(_)))
                    {
                        tokens.push(Self::Operator(Operator::Multiply));
                    }
                    if is_float {
                        current_float.push(c);
                    } else {
                        current_number.push(c);
                    }
                }
                '.' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    if is_float {
                        return Err(ParsingError::new(
                            "Invalid number".to_string(),
                            input.to_string(),
                            i,
                        ));
                    }
                    // if current number is empty,
                    // it means we are parsing a
                    // float starting with a dot
                    if current_number.is_empty() {
                        current_number.push('0');
                    }
                    is_float = true;
                }
                '+' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    compute_number(
                        &mut current_number,
                        &mut current_float,
                        &mut tokens,
                        &mut is_float,
                        i,
                    )?;
                    tokens.push(Self::Operator(Operator::Add));
                }
                '-' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    if current_number.is_empty() {
                        // This is a negative number
                        tokens.push(Self::Operator(Operator::Inverse));
                    } else {
                        compute_number(
                            &mut current_number,
                            &mut current_float,
                            &mut tokens,
                            &mut is_float,
                            i,
                        )?;
                        tokens.push(Self::Operator(Operator::Subtract));
                    }
                }
                '*' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    compute_number(
                        &mut current_number,
                        &mut current_float,
                        &mut tokens,
                        &mut is_float,
                        i,
                    )?;
                    tokens.push(Self::Operator(Operator::Multiply));
                }
                '/' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    compute_number(
                        &mut current_number,
                        &mut current_float,
                        &mut tokens,
                        &mut is_float,
                        i,
                    )?;
                    tokens.push(Self::Operator(Operator::Divide));
                }
                '^' => {
                    if is_parsing_parentesis {
                        continue;
                    }
                    compute_number(
                        &mut current_number,
                        &mut current_float,
                        &mut tokens,
                        &mut is_float,
                        i,
                    )?;
                    tokens.push(Self::Operator(Operator::Pow));
                }
                '(' => {
                    // If previous token is a number,
                    // we add a multiplication operator
                    // and we parse the previous number
                    if !current_number.is_empty() {
                        compute_number(
                            &mut current_number,
                            &mut current_float,
                            &mut tokens,
                            &mut is_float,
                            i,
                        )?;
                        tokens.push(Self::Operator(Operator::Multiply));
                    }
                    if parenthesis_depth == 0 {
                        parenthesis_start = i;
                        is_parsing_parentesis = true;
                    }
                    parenthesis_depth += 1;
                }
                ')' => {
                    parenthesis_depth -= 1;
                    if parenthesis_depth == 0 {
                        // if parenthesis is empty, we return an error
                        if parenthesis_start + 1 == i {
                            return Err(ParsingError::new(
                                "Empty parenthesis".to_string(),
                                input.to_string(),
                                i - 1,
                            ));
                        }
                        is_parsing_parentesis = false;
                        tokens.push(Self::Parenthesis(Self::tokenize(
                            &input[parenthesis_start + 1..i],
                        )?));
                    }
                }
                ' ' => (), // Ignore spaces
                _ => {
                    return Err(ParsingError::new(
                        "Invalid character".to_string(),
                        input.to_string(),
                        i,
                    ))
                }
            }
        }

        if parenthesis_depth != 0 {
            return Err(ParsingError::new(
                "Parenthesis not closed".to_string(),
                input.to_string(),
                input.len() - 1,
            ));
        }

        if !current_number.is_empty() {
            tokens.push(Self::Number(match Self::build_number(&current_number, &current_float) {
                Ok(n) => n,
                Err(_) => {
                    return Err(ParsingError::new(
                        "Invalid number".to_string(),
                        input.to_string(),
                        input.len(),
                    ))
                }
            }));
        }

        Ok(tokens)
    }
}

struct Function {
    signature: String,
    arguments_count: i32,
    function: fn(Vec<f64>) -> f64,
}

impl Function {
    fn new(signature: String, arguments_count: i32, function: fn(Vec<f64>) -> f64) -> Self {
        Self {
            signature,
            arguments_count,
            function,
        }
    }

    fn call(&self, arguments: Vec<f64>) -> f64 {
        (self.function)(arguments)
    }

    fn add() -> Self {
        Self::new(
            "add".to_string(),
            2,
            |arguments: Vec<f64>| arguments[0] + arguments[1],
        )
    }

    fn subtract() -> Self {
        Self::new(
            "sub".to_string(),
            2,
            |arguments: Vec<f64>| arguments[0] - arguments[1],
        )
    }

    fn multiply() -> Self {
        Self::new(
            "mul".to_string(),
            2,
            |arguments: Vec<f64>| arguments[0] * arguments[1],
        )
    }

    fn divide() -> Self {
        Self::new(
            "div".to_string(),
            2,
            |arguments: Vec<f64>| arguments[0] / arguments[1],
        )
    }

    fn inverse() -> Self {
        Self::new(
            "inv".to_string(),
            1,
            |arguments: Vec<f64>| -arguments[0]
        )
    }

    fn pow() -> Self {
        todo!("pow is not implemented yet");
    }


}

enum Token {
    Operator(Function, Vec<Token>),
    Number(f64)
}

impl Token {

    fn new(input: Vec<ParsingToken>) -> Result<Self, ParsingError> {
        // We ord the operators by priority,
        // we store index of the operators
        // ordered by priority. If we have
        // the same priority, we store the
        // order by index (left to right)
        let mut operators = Vec::new();
        for token in &input {
            match token {
                ParsingToken::Operator(o) => {
                    let mut index = 0;
                    for (i, op) in operators.iter().enumerate() {
                        if op < o {
                            index = i + 1;
                        }
                    }
                    operators.insert(index, o);
                }
                _ => (),
            }
        }
        // Now we have the operators ordered by priority
        // we can build the tokens tree easily

    }

    fn compute(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Operator(f, arguments) => f.call(arguments.iter().map(|t| t.compute()).collect())
        }
    }
}

impl std::fmt::Display for ParsingToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Operator(o) => write!(f, " {} ", o),
            Self::Parenthesis(p) => write!(
                f,
                "({})",
                p.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            ),
        }
    }
}

fn display(tokens: &Vec<ParsingToken>) -> Result<i32, &str> {
    println!(
        "{}",
        tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("")
    );
    Ok(0)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if (args.len() > 1) {
        // Merge all input from 1 to ..
        let input = args[1..].join(" ");
        let tokens = match ParsingToken::tokenize(&input) {
            Ok(t) => t,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        display(&tokens).unwrap();
    } else {
        // Token test
        let token = Token::new(ParsingToken::tokenize("1 + 2 * 3").unwrap());
        println!("{}", token.compute());
    }
}
