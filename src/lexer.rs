use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub position: usize,
}

impl SourcePosition {
    pub fn new(line: usize, column: usize, position: usize) -> Self {
        SourcePosition { line, column, position }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub position: SourcePosition,
}

impl LocatedToken {
    pub fn new(token: Token, position: SourcePosition) -> Self {
        LocatedToken { token, position }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String),
    Expression(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<InterpolationPart>),
    Bool(bool),
    
    // Identifiers
    Identifier(String),
    
    // Keywords
    Let,
    Fn,
    Lambda,
    If,
    Else,
    ElseIf,
    While,
    For,
    In,
    Return,
    True,
    False,
    Nil,
    End,
    Do,
    Then,
    Print,
    Import,
    Export,
    From,
    As,
    Match,
    When,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Bang,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    DotDot,
    Colon,
    Arrow,
    DoubleArrow,
    
    // Special
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Identifier(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        
        Lexer {
            input: chars,
            position: 0,
            line: 1,
            column: 1,
            current_char,
        }
    }
    
    fn current_position(&self) -> SourcePosition {
        SourcePosition::new(self.line, self.column, self.position)
    }
    
    fn advance(&mut self) {
        if let Some('\n') = self.current_char {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_multiline_comment(&mut self) {
        // Skip #*
        self.advance();
        self.advance();
        
while let Some(ch) = self.current_char {
    if ch == '*' && self.peek() == Some('#') {
        // Found *#, skip both and exit
        self.advance();
        self.advance();
        return;
    }
    self.advance();
}

// If we reach here, the comment was never closed
// Instead of panicking, we'll just continue (effectively treating as EOF)
    }
    
    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        let mut is_float = false;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek() != Some('.') {
                // Only treat as decimal point if not followed by another dot
                is_float = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        if is_float {
            Token::Float(number.parse().unwrap_or(0.0))
        } else {
            Token::Integer(number.parse().unwrap_or(0))
        }
    }
    
    fn read_string(&mut self) -> Token {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        self.advance(); // Skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => current_text.push('\n'),
                    Some('t') => current_text.push('\t'),
                    Some('r') => current_text.push('\r'),
                    Some('\\') => current_text.push('\\'),
                    Some('"') => current_text.push('"'),
                    Some('$') => current_text.push('$'), // Allow escaping $
                    Some('u') => {
                        // Handle Unicode escape sequences like \u001b
                        self.advance();
                        let mut hex_digits = String::new();
                        for _ in 0..4 {
                            if let Some(hex_char) = self.current_char {
                                if hex_char.is_ascii_hexdigit() {
                                    hex_digits.push(hex_char);
                                    self.advance();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        if hex_digits.len() == 4 {
                            if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
                                if let Some(unicode_char) = std::char::from_u32(code_point) {
                                    current_text.push(unicode_char);
                                } else {
                                    // Invalid Unicode code point, just add the literal characters
                                    current_text.push('\\');
                                    current_text.push('u');
                                    current_text.push_str(&hex_digits);
                                }
                            } else {
                                // Invalid hex, add literal characters
                                current_text.push('\\');
                                current_text.push('u');
                                current_text.push_str(&hex_digits);
                            }
                        } else {
                            // Not enough hex digits, add literal characters
                            current_text.push('\\');
                            current_text.push('u');
                            current_text.push_str(&hex_digits);
                        }
                        continue; // Don't advance again since we already did in the loop
                    }
                    Some(c) => current_text.push(c),
                    None => break,
                }
                self.advance();
            } else if ch == '$' {
                // Start of interpolation
                if !current_text.is_empty() {
                    parts.push(InterpolationPart::Text(current_text.clone()));
                    current_text.clear();
                }
                
                self.advance(); // Skip '$'
                
                if let Some('{') = self.current_char {
                    // ${expression} syntax
                    self.advance(); // Skip '{'
                    let expr = self.read_interpolation_expression();
                    parts.push(InterpolationPart::Expression(expr));
                } else {
                    // $identifier syntax
                    let identifier = self.read_interpolation_identifier();
                    parts.push(InterpolationPart::Expression(identifier));
                }
            } else {
                current_text.push(ch);
                self.advance();
            }
        }
        
        // Add remaining text
        if !current_text.is_empty() {
            parts.push(InterpolationPart::Text(current_text));
        }
        
        // If no interpolation parts, return regular string
        if parts.len() == 1 {
            if let InterpolationPart::Text(text) = &parts[0] {
                return Token::String(text.clone());
            }
        }
        
        // Return interpolated string if we have parts
        if parts.is_empty() {
            Token::String(String::new())
        } else {
            Token::InterpolatedString(parts)
        }
    }
    
    fn read_interpolation_expression(&mut self) -> String {
        let mut expr = String::new();
        let mut brace_count = 1;
        
        while let Some(ch) = self.current_char {
            if ch == '{' {
                brace_count += 1;
                expr.push(ch);
                self.advance();
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    self.advance(); // Skip closing '}'
                    break;
                } else {
                    expr.push(ch);
                    self.advance();
                }
            } else {
                expr.push(ch);
                self.advance();
            }
        }
        
        expr
    }
    
    fn read_interpolation_identifier(&mut self) -> String {
        let mut identifier = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        identifier
    }
    
    fn read_identifier(&mut self) -> Token {
        let mut identifier = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        match identifier.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "lambda" => Token::Lambda,
            "if" => Token::If,
            "else" => Token::Else,
            "elseif" => Token::ElseIf,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            "nil" => Token::Nil,
            "end" => Token::End,
            "do" => Token::Do,
            "then" => Token::Then,
            "print" => Token::Print,
            "import" => Token::Import,
            "export" => Token::Export,
            "from" => Token::From,
            "as" => Token::As,
            "and" => Token::And,
            "or" => Token::Or,
            "match" => Token::Match,
            "when" => Token::When,
            _ => Token::Identifier(identifier),
        }
    }
    
    pub fn next_token(&mut self) -> LocatedToken {
        loop {
            let start_pos = self.current_position();
            match self.current_char {
                None => return LocatedToken::new(Token::Eof, start_pos),
                Some(' ') | Some('\t') | Some('\r') => {
                    self.skip_whitespace();
                    continue;
                }
                Some('\n') => {
                    self.advance();
                    return LocatedToken::new(Token::Newline, start_pos);
                }
                Some('#') => {
                    if self.peek() == Some('*') {
                        self.skip_multiline_comment();
                    } else {
                        self.skip_comment();
                    }
                    continue;
                }
                Some('+') => {
                    self.advance();
                    return LocatedToken::new(Token::Plus, start_pos);
                }
                Some('-') => {
                    if self.peek() == Some('>') {
                        self.advance();
                        self.advance();
                        return LocatedToken::new(Token::Arrow, start_pos);
                    }
                    self.advance();
                    return LocatedToken::new(Token::Minus, start_pos);
                }
                Some('*') => {
                    self.advance();
                    return LocatedToken::new(Token::Star, start_pos);
                }
                Some('/') => {
                    self.advance();
                    return LocatedToken::new(Token::Slash, start_pos);
                }
                Some('%') => {
                    self.advance();
                    return LocatedToken::new(Token::Percent, start_pos);
                }
                Some('=') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return LocatedToken::new(Token::EqualEqual, start_pos);
                    }
                    self.advance();
                    return LocatedToken::new(Token::Equal, start_pos);
                }
                Some('!') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return LocatedToken::new(Token::BangEqual, start_pos);
                    }
                    self.advance();
                    return LocatedToken::new(Token::Bang, start_pos);
                }
                Some('<') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return LocatedToken::new(Token::LessEqual, start_pos);
                    }
                    return LocatedToken::new(Token::Less, start_pos);
                }
                Some('>') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return LocatedToken::new(Token::GreaterEqual, start_pos);
                    }
                    return LocatedToken::new(Token::Greater, start_pos);
                }
                Some('(') => {
                    self.advance();
                    return LocatedToken::new(Token::LeftParen, start_pos);
                }
                Some(')') => {
                    self.advance();
                    return LocatedToken::new(Token::RightParen, start_pos);
                }
                Some('[') => {
                    self.advance();
                    return LocatedToken::new(Token::LeftBracket, start_pos);
                }
                Some(']') => {
                    self.advance();
                    return LocatedToken::new(Token::RightBracket, start_pos);
                }
                Some('{') => {
                    self.advance();
                    return LocatedToken::new(Token::LeftBrace, start_pos);
                }
                Some('}') => {
                    self.advance();
                    return LocatedToken::new(Token::RightBrace, start_pos);
                }
                Some(',') => {
                    self.advance();
                    return LocatedToken::new(Token::Comma, start_pos);
                }
                Some('.') => {
                    if self.peek() == Some('.') {
                        self.advance();
                        self.advance();
                        return LocatedToken::new(Token::DotDot, start_pos);
                    }
                    self.advance();
                    return LocatedToken::new(Token::Dot, start_pos);
                }
                Some(':') => {
                    self.advance();
                    return LocatedToken::new(Token::Colon, start_pos);
                }
                Some('"') => {
                    return LocatedToken::new(self.read_string(), start_pos);
                }
                Some(ch) if ch.is_ascii_digit() => {
                    return LocatedToken::new(self.read_number(), start_pos);
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    return LocatedToken::new(self.read_identifier(), start_pos);
                }
                Some(ch) => {
                    println!("Unexpected character: {}", ch);
                    self.advance();
                    continue;
                }
            }
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        loop {
            let located_token = self.next_token();
            let is_eof = matches!(located_token.token, Token::Eof);
            tokens.push(located_token.token);
            if is_eof {
                break;
            }
        }
        
        tokens
    }
    
    pub fn tokenize_with_positions(&mut self) -> Vec<LocatedToken> {
        let mut tokens = Vec::new();
        
        loop {
            let located_token = self.next_token();
            let is_eof = matches!(located_token.token, Token::Eof);
            tokens.push(located_token);
            if is_eof {
                break;
            }
        }
        
        tokens
    }
}
