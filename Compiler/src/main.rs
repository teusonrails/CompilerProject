/* Lexer */
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Let,
    Function,
    Return,
    Ident(String),
    Number(i32),
    FloatLiteral(f64),
    StringLiteral(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    Assign,
    EqualEqual,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    IntType,
    FloatType,
    BoolType,
    StringType,
    Else,
    If,
    Eof,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: String) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Token::Eof;
        }

        let ch = self.input[self.position];
        
        let token = match ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.position += 1;
                    Token::EqualEqual
                } else {
                    Token::Assign
                }
            },
            '!' => {
                if self.peek_char() == '=' {
                    self.position += 1;
                    Token::NotEqual
                } else {
                    panic!("Caractere desconhecido: {}", ch);
                }
            },
            '>' => {
                if self.peek_char() == '=' {
                    self.position += 1;
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            },
            '<' => {
                if self.peek_char() == '=' {
                    self.position += 1;
                    Token::LessEqual
                } else {
                    Token::Less
                }
            },
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            ';' => Token::Semicolon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '"' => return self.read_string(),
            _ => {
                if ch.is_digit(10) {
                    return self.read_number();
                } else if ch.is_alphabetic() {
                    return self.read_identifier();
                } else {
                    panic!("Caracter desconhecido: {}", ch);
                }
            }
        };

        self.position += 1;
        token
    }

    fn peek_char(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn skip_whitespace(&mut self){
        while self.position < self.input.len() && self.input[self.position].is_whitespace(){
            self.position += 1;
        }
    }

    fn read_number(&mut self) -> Token{
        let start = self.position;
        let mut has_dot = false;
        while self.position < self.input.len() && (self.input[self.position].is_digit(10) || (self.input[self.position] == '.' && !has_dot)){
            if self.input[self.position] == '.' {
                has_dot = true;
            }
            self.position += 1
        }
        
        let number: String = self.input[start..self.position].iter().collect();
        
        if has_dot {
            let float_value: f64 = number.parse().unwrap();
            Token::FloatLiteral(float_value)
        } else {
            let int_value: i32 = number.parse().unwrap();
            Token::Number(int_value)
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_alphanumeric(){
            self.position += 1;
        }
        let ident: String = self.input[start..self.position].iter().collect();

        match ident.as_str() {
            "int" => Token::IntType,
            "float" => Token::FloatType,
            "bool" => Token::BoolType,
            "string" => Token::StringType,
            "let" => Token::Let,
            "function" => Token::Function,
            "return" => Token::Return,
            "if" => Token::If,
            "Else" => Token::Else,
            _ => Token::Ident(ident),
        }
    }

    fn read_string(&mut self) -> Token {
        self.position += 1;
        let start = self.position;

        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }

        if self.position >= self.input.len() {
            panic!("String não terminada!");
        }

        let string_value: String = self.input[start..self.position].iter().collect();
        self.position += 1;
        Token::StringLiteral(string_value)
    }
}

/* Parser */
#[derive(Debug, Clone)]
enum ASTNode {
    Program(Vec<Box<ASTNode>>),
    LetDeclaration {
        identifier: String,
        var_type: Type,
        expression: Box<ASTNode>,
    },
    BinaryExpression {
        left: Box<ASTNode>,
        operator: Token,
        right: Box<ASTNode>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Box<ASTNode>>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Box<ASTNode>>,
    },
    Return {
        expression: Box<ASTNode>,
    },
    IfElse {
        condition: Box<ASTNode>,
        if_branch: Vec<Box<ASTNode>>,
        else_branch: Option<Vec<Box<ASTNode>>>,
    },
    Identifier(String),
    Number(i32),
    FloatLiteral(f64),
    BooleanLiteral(bool),
    StringLiteral(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Int,
    Float, 
    Bool,
    String,
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn parse_program(&mut self) -> ASTNode {
        let mut nodes = Vec::new();

        while self.current_token() != Token::Eof {
            let node = self.parse_statement();
            nodes.push(Box::new(node));
        }

        ASTNode::Program(nodes)
    }

    fn parse_statement(&mut self) -> ASTNode {
        match self.current_token() {
            Token::Let => self.parse_let_declaration(),
            Token::Function => self.parse_function_declaration(),
            Token::Return => self.parse_return_declaration(),
            Token::If => self.parse_if_statement(),
            _ => self.parse_expression(),
        }
    }

    fn parse_let_declaration(&mut self) -> ASTNode {
        self.eat(Token::Let);

        let identifier = match self.current_token() {
            Token::Ident(name) => name.clone(),
            _ => panic!("Esperando identificador após 'let'"),
        };
        self.eat(Token::Ident(identifier.clone()));

        self.eat(Token::Colon);

        let var_type = match self.current_token() {
                Token::IntType => {
                    self.eat(Token::IntType);
                    Type::Int
                },
                Token::FloatType => {
                    self.eat(Token::FloatType);
                    Type::Float
                },
                Token::BoolType => {
                    self.eat(Token::BoolType);
                    Type::Bool
                },
                Token::StringType => {
                    self.eat(Token::BoolType);
                    Type::String
                },
                _ => panic!("Esperado tipo após ':'"),
            };

        self.eat(Token::Assign);
        let expression = self.parse_expression();
        self.eat(Token::Semicolon);

        ASTNode::LetDeclaration {
            identifier,
            var_type,
            expression: Box::new(expression),
        }
    }

    fn parse_expression(&mut self) -> ASTNode {
        self.parse_additive_expression()
    }

    fn parse_additive_expression(&mut self) -> ASTNode {
        let mut left = self.parse_term();

        while self.current_token() == Token::Plus || self.current_token() == Token::Minus {
            let operator = self.current_token();
            self.eat(operator.clone());

            let right = self.parse_term();
            left = ASTNode::BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_term(&mut self) -> ASTNode {
        let mut left = self.parse_factor();

        while self.current_token() == Token::Multiply || self.current_token() == Token::Divide {
            let operator = self.current_token();
            self.eat(operator.clone());

            let right = self.parse_factor();
            left = ASTNode::BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_factor(&mut self) -> ASTNode {
        match self.current_token() {
            Token::Number(value) => {
                self.eat(Token::Number(value));
                ASTNode::Number(value)
            }
            Token::FloatLiteral(value) => {
                self.eat(Token::FloatLiteral(value));
                ASTNode::FloatLiteral(value)
            }
            Token::StringLiteral(value) => {
                self.eat(Token::StringLiteral(value.clone()));
                ASTNode::StringLiteral(value)
            }
            Token::Ident(name) => {
                let identifier = name.clone();
                self.eat(Token::Ident(name));

                if self.current_token() == Token::LParen {
                    return self.parse_function_call(identifier);
                }

                ASTNode::Identifier(identifier)
            }
            Token::LParen => {
                self.eat(Token::LParen);
                let expr = self.parse_expression();
                self.eat(Token::RParen);
                expr
            }
            _ => panic!("Token inesperado ao tentar parsear fator"),
        }
    }

    fn parse_function_declaration(&mut self) -> ASTNode {
        self.eat(Token::Function);

        let name = match self.current_token() {
            Token::Ident(name) => name.clone(),
            _ => panic!("Esperado identificador após 'function'"),
        };
        self.eat(Token::Ident(name.clone()));

        self.eat(Token::LParen);

        let mut parameters = Vec::new();
        while self.current_token() != Token::RParen {
            if let Token::Ident(param) = self.current_token() {
                parameters.push(param.clone());
                self.eat(Token::Ident(param));
            }

            if self.current_token() == Token::Comma {
                self.eat(Token::Comma);
            }
        }
        self.eat(Token::RParen);

        self.eat(Token::LBrace);

        let mut body = Vec::new();
        while self.current_token() != Token::RBrace {
            let stmt = self.parse_statement();
            body.push(Box::new(stmt));
        }
        self.eat(Token::RBrace);

        ASTNode::FunctionDeclaration {
            name,
            parameters,
            body,
        }
    }

    fn parse_return_declaration(&mut self) -> ASTNode {
        self.eat(Token::Return);
        let expression = self.parse_expression();
        self.eat(Token::Semicolon);
        ASTNode::Return {
            expression: Box::new(expression),
        }
    }

    fn parse_function_call(&mut self, name: String) -> ASTNode {
        self.eat(Token::LParen);

        let mut arguments = Vec::new();
        while self.current_token() != Token::RParen {
            let argument = self.parse_expression();
            arguments.push(Box::new(argument));

            if self.current_token() == Token::Comma {
                self.eat(Token::Comma);
            }
        }
        self.eat(Token::RParen);

        ASTNode::FunctionCall { name, arguments }
    }
    fn parse_if_statement(&mut self) -> ASTNode {
        self.eat(Token::If);

        self.eat(Token::LParen);
        let condition = self.parse_expression();
        self.eat(Token::RParen);

        self.eat(Token::LBrace);
        let if_branch = self.parse_block();
        self.eat(Token::RBrace);
        
        let else_branch = if self.current_token() == Token::Else {
            self.eat(Token::Else);
            self.eat(Token::LBrace);
            let block = self.parse_block();

            self.eat(Token::RBrace);
            Some(block)
        } else {
            None
        };

        ASTNode::IfElse {
            condition: Box::new(condition),
            if_branch,
            else_branch,
        }
    }

    fn parse_block(&mut self) -> Vec<Box<ASTNode>> {
        let mut statements = Vec::new();

        while self.current_token() != Token::RBrace && self.current_token() != Token::Eof {
            let statement = self.parse_statement();

            statements.push(Box::new(statement));
        }
        statements
    }

    fn parse_comparison_expression(&mut self) -> ASTNode {
        let mut left = self.parse_additive_expression();

        while matches!(
            self.current_token(),
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual | Token::EqualEqual | Token::NotEqual
        ) {
            let operator = self.current_token();
            self.eat(operator.clone());

            let right = self.parse_additive_expression();
            left = ASTNode::BinaryExpression {
                left: Box::new(left);
                operator,
                right: Box::new(right),
            };
        }
    }

    fn eat(&mut self, expected: Token) {
        if self.current_token() == expected {
            self.position += 1;
        } else {
            panic!("Esperado token {:?}, encontrado {:?}", expected, self.current_token())
        }
    }

    fn current_token(&self) -> Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof).clone()
    }

}

/* Virtual Machine */
use std::fmt;
#[derive(Debug, Clone, PartialEq)]
enum Value{
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
}

use std::collections::HashMap;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
        }
    }
}

struct MiniLangVM {
    variables: HashMap<String, (Value, Type)>,
    variable_order: Vec<String>,
    functions: HashMap<String, (Vec<String>, Vec<Box<ASTNode>>)>,
}

impl MiniLangVM {
    #![allow(unused_variables)]

    fn assert_variable(&self, name: &str, expected_value: Value, expected_type: Type) {
        match self.variables.get(name) {
            Some((value, value_type)) if *value == expected_value && *value_type == expected_type => {
                println!("Teste bem-sucedido: {} = {:?}", name, expected_value);
            }
            Some((value, expr_type)) => {
                println!("Teste falhou: Esperado {:?}, mas encontrou {:?}", expected_value, value);
            }
            None => {
                println!("Teste falhou: Variavel, não encontrada!");
            }
        }
    }

    fn new() -> Self {
        MiniLangVM {
            variables: HashMap::new(),
            variable_order: Vec::new(),
            functions: HashMap::new(),
        }
    }

    fn execute_function_call(&mut self, name:String, arguments: Vec<Box<ASTNode>>) -> Option<(Value, Type)> {
        let (parameters, body) = match self.functions.get(&name) {
            Some(func) => (func.0.clone(), func.1.clone()),
            None => panic!("Função '{}' não encontrada", name),
        };

        let mut local_variables = HashMap::new();

        for (param, arg) in parameters.iter().zip(arguments) {
            let (value, expr_type) = self.evaluate_expression(*arg);
            local_variables.insert(param.clone(), (value, expr_type));
        }

        let global_variables_backup = self.variables.clone();
        self.variables = local_variables;
        
        for statement in body {
            if let Some((ret_val, ret_type)) = self.run(*statement) {
                self.variables = global_variables_backup;
                return Some((ret_val, ret_type));
            }
        }

        self.variables = global_variables_backup;
        None
    }

    fn determine_type(&self, value: &Value) -> Type {
        match value {
            Value::Int(value) => Type::Int,
            Value::Float(value) => Type::Float,
            Value::Bool(value) => Type::Bool,
            Value::String(value) => Type::String,
        }
    }

    fn evaluate_expression(&mut self, expression: ASTNode) -> (Value, Type) {
        match expression {
            ASTNode::Number(value) => (Value::Int(value), Type::Int),
            ASTNode::FloatLiteral(value) => (Value::Float(value), Type::Float),
            ASTNode::BooleanLiteral(value) => (Value::Bool(value), Type::Bool),
            ASTNode::StringLiteral(value) => (Value::String(value), Type::String),
            ASTNode::Identifier(name) => {
                let value = self.variables.get(&name).expect("Variável não encontrada!").clone();
                let expr_type = self.determine_type(&value.0);

                (value.0, Type::Int)
            }
            ASTNode::BinaryExpression {left, operator, right} => {
                let (left_value, left_type) = self.evaluate_expression(*left);
                let (right_value, right_type) = self.evaluate_expression(*right);

                if left_type != right_type {
                    panic!("Erro de tipo: Não é permitido misturar tipos diferentes em operações!")
                }

                let result = match (left_value, right_value, operator) {
                    (Value::Int(l), Value::Int(r), Token::Plus) => (Value::Int(l + r), Type::Int),
                    (Value::Int(l), Value::Int(r), Token::Minus) => (Value::Int(l - r), Type::Int),
                    (Value::Int(l), Value::Int(r), Token::Multiply) => (Value::Int(l * r), Type::Int),
                    (Value::Int(l), Value::Int(r), Token::Divide) => (Value::Int(l / r), Type::Int),

                    (Value::Float(l), Value::Float(r), Token::Plus) => (Value::Float(l + r), Type::Float),
                    (Value::Float(l), Value::Float(r), Token::Minus) => (Value::Float(l - r), Type::Float),
                    (Value::Float(l), Value::Float(r), Token::Multiply) => (Value::Float(l * r), Type::Float),
                    (Value::Float(l), Value::Float(r), Token::Divide) => (Value::Float(l / r), Type::Float),

                    (Value::String(l), Value::String(r), Token::Plus) => (Value::String(l + &r), Type::String),

                    (Value::Int(l), Value::Int(r), Token::EqualEqual) => (Value::Bool(l == r), Type::Bool),
                    (Value::Int(l), Value::Int(r), Token::NotEqual) => (Value::Bool(l != r), Type::Bool),
                    (Value::Int(l), Value::Int(r), Token::Greater) => (Value::Bool(l > r), Type::Bool),
                    (Value::Int(l), Value::Int(r), Token::GreaterEqual) => (Value::Bool(l >= r), Type::Bool),
                    (Value::Int(l), Value::Int(r), Token::Less) => (Value::Bool(l < r), Type::Bool),
                    (Value::Int(l), Value::Int(r), Token::LessEqual) => (Value::Bool(l <= r), Type::Bool),
                    
                    (Value::Float(l), Value::Float(r), Token::EqualEqual) => (Value::Bool(l == r), Type::Bool),
                    (Value::Float(l), Value::Float(r), Token::NotEqual) => (Value::Bool(l != r), Type::Bool),
                    (Value::Float(l), Value::Float(r), Token::Greater) => (Value::Bool(l > r), Type::Bool),
                    (Value::Float(l), Value::Float(r), Token::GreaterEqual) => (Value::Bool(l >= r), Type::Bool),
                    (Value::Float(l), Value::Float(r), Token::Less) => (Value::Bool(l < r), Type::Bool),
                    (Value::Float(l), Value::Float(r), Token::LessEqual) => (Value::Bool(l <= r), Type::Bool),

                    _ => panic!("Erro de tipo: Operação não suportada para esses tipos!"),
                };
                
                result
            }
        
            ASTNode::FunctionCall { name, arguments } => {
                if let Some((result_value, result_type)) = self.execute_function_call(name, arguments){
                    (result_value, result_type)
                } else {
                    panic!("Função não retornou nenhum valor!");
                }
            }
            ASTNode::FunctionDeclaration { name, parameters, body } => {
                panic!("Declaração de função encontrada em um contexto inesperado!");
            }
            ASTNode::Return {expression} => {
                let (value, expr_type) = self.evaluate_expression(*expression);
                (value, expr_type)
            }
            _ => panic!("Expressão não suportada!"),
        }
    }

    fn execute_if_else(&mut self, condition: ASTNode, if_branch: Vec<Box<ASTNode>>, else_branch: Option<Vec<Box<ASTNode>>>) {
        let (condition_value, _) = self.evaluate_expression(condition);

        match condition_value {
            Value::Bool(true) => {
                for stmt in if_branch {
                    self.run(*stmt);
                }
            }
            Value::Bool(true) => {
                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.run(*stmt);
                    }
                }
            }
            _ => panic!("Erro de tipo: Condição em um 'if' deve ser booleano!"),
        }
    }

    fn run(&mut self, ast: ASTNode) -> Option<(Value, Type)> {
        match ast {
            ASTNode::Program(statements) => {
                for statement in statements {
                    if let Some((value, value_type)) = self.run(*statement) {
                        return Some((value, value_type));
                    }
                }
            }
            ASTNode::LetDeclaration {identifier, var_type, expression} => {
                let (value, expr_type) = self.evaluate_expression(*expression);

                if var_type != expr_type {
                    panic!("Erro de tipo: Esperado {:?}, mas obteve {:?}", var_type, expr_type);
                }

                self.variables.insert(identifier.clone(), (value, expr_type));
                self.variable_order.push(identifier);
            }
            ASTNode::FunctionDeclaration {name, parameters, body} => {
                self.functions.insert(name, (parameters, body));
            }
            ASTNode::FunctionCall {name, arguments} => {
               if let Some((result_value, result_type)) = self.execute_function_call(name, arguments) {
                return Some((result_value, result_type));
               } else {
                panic!("Função não retornou nenhum valor!");
               }
            }
            ASTNode::Return {expression} => {
                let (value, expr_type) = self.evaluate_expression(*expression);
                return Some((value, expr_type ));
            }
            ASTNode::IfElse { condition, if_branch, else_branch } => {
                self.execute_if_else(*condition, if_branch, else_branch);
            }
            _ => panic!("Instrução não suportada!"),
        }
        None
    }
    
    fn print_environment(&self) {
        println!("Ambiente de Execução:");

        for (name, (value, value_type)) in &self.variables {
            println!("{} = {:?} ({:?})", name, value, value_type);
        }
    }
}


fn main() {
    let input = String::from("
    let x: int = 10;
    let y: int = 20;

    if (x < y) {
        let z: int = x + y;
    } else {
        let z: int = x - y;
    }

    let a: bool = x > y;

    if (a) {
        let resultado: string = \"x é maior que y\";
    } else {
        let resultado: string = \"x não é maior que y\";
    }
    ");

    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        println!("{:?}", token);
        tokens.push(token.clone());
        if token == Token::Eof {
            break;
        }
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    println!("{:?}", ast);

    let mut vm = MiniLangVM::new();
    vm.run(ast);

    for var_name in &vm.variable_order {
        if let Some((value, _value_type)) = vm.variables.get(var_name) {
            println!("{} = {:?}", var_name, value);
        }
    }
    vm.print_environment();

    if let Some((value, _value_type)) = vm.variables.get("x") {
        println!("O valor da variável x é: {:?}", value);
    } else {
        println!("A variável x não foi encontrada.");
    }
}