#[derive(Debug, PartialEq, Clone)]
pub enum TokenT {
    Literal(String),
    Number(f64),
    Operator(Operator),
    OpenParen,
    CloseParen,
    Dot,
    Match,
}

#[derive(Debug, Clone)]
pub struct Token {
    ty: TokenT,
    idx: usize,
}

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Literal(String),
    Number(f64),
    BinaryOp {
        op: Operator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Match(Box<ASTNode>),
    Unexpected
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operator {
    GTE,
    GT,
    EQ,
    NEQ,
    LT,
    LTE
}

#[derive(Debug)]
pub enum ParseErrorT {
    RHSofOperatorMustBeLiteralOrNumber,
    NoDotBetweenFns,
    MatchFnArgsMalformed,
    UnexpectedToken(Token)
}

#[derive(Debug)]
pub struct ParseError {
    pub ty: ParseErrorT,
    pub cursor: usize
}

pub struct Crawler {
    s: String,
    pub ast: Vec<ASTNode>,
}

impl Crawler {
    pub fn new(s: String) -> Self {
        Self {
            s,
            ast: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Result<(), ParseError> {
        let tokens = Crawler::tokenize(&self.s);
        self.parse_tokens(&tokens)
    }

    fn tokenize(s: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = s.chars().peekable();
        let mut idx: usize = 0;
        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\t' | '\n' => {
                    chars.next();
                    idx+=1;
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Operator(Operator::GTE), idx});
                        idx+=2;
                    } else {
                        tokens.push(Token {ty: TokenT::Operator(Operator::GT), idx});
                        idx+=1;
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Operator(Operator::LTE), idx});
                        idx+=2;
                    } else {
                        tokens.push(Token {ty: TokenT::Operator(Operator::LT), idx});
                        idx+=1;
                    }
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Operator(Operator::EQ), idx});
                        idx+=2;
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Operator(Operator::NEQ), idx});
                        idx+=2;
                    }
                }
                '(' => {
                    tokens.push(Token {ty: TokenT::OpenParen, idx});
                    chars.next();
                    idx+=1;
                }
                ')' => {
                    tokens.push(Token {ty: TokenT::CloseParen, idx});
                    chars.next();
                    idx+=1;
                }
                '.' => {
                    tokens.push(Token {ty: TokenT::Dot, idx});
                    chars.next();
                    idx+=1;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut literal = String::new();
                    while let Some(&c1) = chars.peek() {
                        if c1.is_alphanumeric() || c1 == '_' || c1 == '.' {
                            literal.push(c1);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    if literal == "match" {
                        tokens.push(Token { ty: TokenT::Match, idx});
                        idx+=5;
                    } else {
                        let literal_size = literal.len();
                        tokens.push(Token { ty: TokenT::Literal(literal), idx});
                        idx+=literal_size;
                    }
                }
                '0'..='9' => {
                    let mut number = String::new();
                    while let Some(&c1) = chars.peek() {
                        if c1.is_numeric() || c1 == '.' {
                            number.push(c1);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let number_size = number.len();
                    tokens.push(Token { ty: TokenT::Number(number.parse().unwrap()), idx});
                    idx+=number_size;
                }
                _ => {
                    panic!("unexpected character while parsing string: {}", c);
                }
            }
        }
        tokens
    }

    pub fn parse_tokens(&mut self, tokens: &Vec<Token>) -> Result<(), ParseError>{
        let mut nodes = Vec::new();
        let mut iter = tokens.iter().peekable();
        while let Some(t) = iter.next() {
            match t.ty {
                TokenT::Match => {
                    iter.next(); // skip OpenParen
                    if let (
                        Some(Token{ty: TokenT::Literal(l1), idx: l1_idx}),
                        Some(Token{ty: TokenT::Operator(op), idx: op_idx}),
                        Some(Token{ty: next, idx: next_idx})
                    ) = (iter.next(), iter.next(), iter.peek()) {
                        let right = match next {
                            TokenT::Literal(l2) => {
                                iter.next();
                                ASTNode::Literal(l2.clone())
                            }
                            TokenT::Number(n1) => {
                                iter.next();
                                ASTNode::Number(*n1)
                            }
                            _ => ASTNode::Unexpected,
                        };
                        if right == ASTNode::Unexpected {
                            return Err(ParseError {ty: ParseErrorT::RHSofOperatorMustBeLiteralOrNumber, cursor: *next_idx});
                        }
                        nodes.push(ASTNode::Match(Box::new(ASTNode::BinaryOp {
                            op: op.clone(),
                            left: Box::new(ASTNode::Literal(l1.clone())),
                            right: Box::new(right),
                        })));
                        iter.next(); // skip CloseParen
                        let dot_check = iter.next();
                        match dot_check {
                            Some(Token {ty: TokenT::Dot, ..}) | None => {},
                            other => return Err(ParseError {ty: ParseErrorT::NoDotBetweenFns, cursor: dot_check.unwrap().idx}),
                        }
                    } else {
                        return Err(ParseError {ty: ParseErrorT::MatchFnArgsMalformed, cursor: t.idx});
                    }
                }
                _ => return Err(ParseError {ty: ParseErrorT::UnexpectedToken(t.clone()), cursor: t.idx}),
            }
        }
        self.ast = nodes;
        Ok(())
    }
}