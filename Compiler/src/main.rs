/* Lexer */
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Let,
    Function,
    Return,
    Ident(String),
    Number(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
    Assign,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    IntType,
    FloatType,
    BoolType,
    StringType,
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
            '=' => Token::Assign,
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

    fn skip_whitespace(&mut self){
        while self.position < self.input.len() && self.input[self.position].is_whitespace(){
            self.position += 1;
        }
    }

    fn read_number(&mut self) -> Token{
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_digit(10) {
            self.position += 1
        }
        let number: i32 = self.input[start..self.position].iter().collect::<String>().parse().unwrap();
        Token::Number(number)
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
            _ => Token::Ident(ident),
        }
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
    Identifier(String),
    Number(i32),
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

        let var_type = match self.current_token(){
            Token::IntType => Type::Int,
            Token::FloatType => Type::Float,
            Token::BoolType => Type::Bool,
            Token::StringType => Type::String,
            _ => panic!("Esperado tipo após ':'"),
        };
        self.eat(self.current_token().clone());

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
#[derive(Debug, Clone)]
enum Value{
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
}

use std::collections::HashMap;


struct MiniLangVM {
    variables: HashMap<String, Value>,
    variable_order: Vec<String>,
    functions: HashMap<String, (Vec<String>, Vec<Box<ASTNode>>)>,
}

impl MiniLangVM {
    #![allow(unused_variables)]

    fn assert_variable(&self, name: &str, expected_value: Value) {
        match self.variables.get(name) {
            Some(value) if *value == expected_value => {
                println!("Teste bem-sucedido: {} = {:?}", name, expected_value);
            }
            Some(value) => {
                println!("Teste falhou: Esperado {:?}, mas encontrou {:?}", expected_value, value);
            }
            None => {
                println!("Teste falhou: Variavel '{}', não encontrada!");
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

    fn execute_function_call(&mut self, name:String, arguments: Vec<Box<ASTNode>>) -> Option<i32> {
        let (parameters, body) = match self.functions.get(&name) {
            Some(func) => (func.0.clone(), func.1.clone()),
            None => panic!("Função '{}' não encontrada", name),
        };

        let mut local_variables = HashMap::new();

        for (param, arg) in parameters.iter().zip(arguments) {
            let value = self.evaluate_expression(*arg);
            local_variables.insert(param.clone(), value);
        }

        let global_variables_backup = self.variables.clone();

        self.variables = local_variables;

        let result = None;
        for statement in body {
            if let Some(ret_val) = self.run(*statement) {
                return Some(ret_val);
            }
        }

        self.variables = global_variables_backup;

        result
    }

    fn evaluate_expression(&mut self, expression: ASTNode) -> (i32, Type) {
        match expression {
            ASTNode::Number(value) => Value::Int(value),
            ASTNode::FloatLiteral(value) => Value::Float(value),
            ASTNode::BooleanLiteral(value) => Value::Bool(value),
            ASTNode::StringLiteral(value) => Value::String(value),
            ASTNode::Identifier(name) => {
                *self.variables.get(&name).expect("Variável não encontrada!").clone()
            }
            ASTNode::BinaryExpression {left, operator, right} => {
                let left_value = self.evaluate_expression(*left);
                let right_value = self.evaluate_expression(*right);

                match (left_value, right_value, operator) {
                    (Value::Int(l), Value::Int(r), Token::Plus) => Value::Int(l + r),
                    (Value::Int(l), Value::Int(r), Token::Minus) => Value::Int(l - r),
                    (Value::Int(l), Value::Int(r), Token::Multiply) => Value::Int(l * r),
                    (Value::Int(l), Value::Int(r), Token::Divide) => Value::Int(l / r),

                    (Value::Float(l), Value::Float(r), Token::Plus) => Value::Float(l + r),
                    (Value::Float(l), Value::Float(r), Token::Minus) => Value::Float(l - r),
                    (Value::Float(l), Value::Float(r), Token::Multiply) => Value::Float(l * r),
                    (Value::Float(l), Value::Float(r), Token::Divide) => Value::Float(l / r),

                    (Value::String(1), Value::String(1), Token::Plus) => {
                        Value::String(1 + &r)
                    }

                    _ => panic!("Erro de tipo: Operação não suportada para esses tipos!"),
                }
            }
            ASTNode::FunctionCall { name, arguments } => {
                if let Some(result) = self.execute_function_call(name, arguments){
                    result
                } else {
                    panic!("Função não retornou nenhum valor!");
                }
            }
            ASTNode::FunctionDeclaration { name, parameters, body } => {
                panic!("Declaração de função encontrada em um contexto inesperado!");
            }
            ASTNode::Return {expression} => {
                let value = self.evaluate_expression(*expression);
                return value;
            }
            _ => panic!("Expressão não suportada!"),
        }
    }
    fn run(&mut self, ast: ASTNode) -> Option<i32> {
        match ast {
            ASTNode::Program(statements) => {
                for statement in statements {
                    if let Some(value) = self.run(*statement) {
                        return Some(value);
                    }
                }
            }
            ASTNode::LetDeclaration {identifier,var_type , expression} => {
                let (value, expr_type) = self.evaluate_expression(*expression);

                if var_type != expr_type {
                    panic!("Erro de tipo: Esperado {:?}, mas obteve {:?}", var_type, expr_type);
                }

                self.variables.insert(identifier.clone(), value);
                self.variable_order.push(identifier);
            }
            ASTNode::FunctionDeclaration {name, parameters, body} => {
                self.functions.insert(name, (parameters, body));
            }
            ASTNode::FunctionCall {name, arguments} => {
               return self.execute_function_call(name, arguments);
            }
            ASTNode::Return {expression} => {
                let value = self.evaluate_expression(*expression);
                return Some(value);
            }
            _ => panic!("Instrução não suportada!"),
        }
        None
    }
    
    fn print_environment(&self) {
        println!("Ambiente de Execução:");

        for (name, value) in &self.variables {
            println!("{} = {:?}", name, value);
        }
    }
}


fn main() {
    let input = String::from("
        function soma(a, b) {
            return a + b;
        }

        let x = 10;
        let y = 20;
        let z = soma(x, y);
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
        if let Some(value) = vm.variables.get(var_name) {
            println!("{} = {}", var_name, value);
        }
    }
    vm.print_environment();

    if let Some(value) = vm.variables.get("x") {
        println!("O valor da variável x é: {:?}", value);
    } else {
        println!("A variável x não foi encontrada.");
    }
}