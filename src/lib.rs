use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenT {
    Literal(String),
    Number(f64),
    Comparator(Comparator),
    OpenParen,
    CloseParen,
    Dot,
    Match,
    ConditionalOperator(ConditionalOperator)
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
    Condition {
        op: Comparator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    ConditionalOperator {
        op: ConditionalOperator,
        conditions: Vec<Box<ASTNode>>
    },
    Match(Box<ASTNode>),
    Unexpected
}

impl std::ops::Deref for ASTNode {
    type Target = ASTNode;

    fn deref(&self) -> &Self::Target {
        match self {
            ASTNode::Match(inner) => &**inner,
            _ => self,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Comparator {
    GTE,
    GT,
    EQ,
    NEQ,
    LT,
    LTE
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ConditionalOperator {
    AND,
    OR
}

#[derive(Debug)]
pub enum ParseErrorT {
    RHSofComparatorMustBeLiteralOrNumber,
    NoDotBetweenFns,
    InvalidBinopStructure,
    Unexpected, // TODO: add the token that is unexpected later
    UnmatchedParenthesis,
    MissingComparator, 
    MissingOpenParen
}

#[derive(Debug)]
pub struct ParseError {
    pub ty: ParseErrorT,
    pub cursor: usize
}

pub struct MonGod {
    s: String,
    pub ast: Vec<ASTNode>,
}

impl MonGod {
    pub fn new(s: String) -> Self {
        Self {
            s,
            ast: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Result<(), ParseError> {
        let tokens = MonGod::tokenize(&self.s);
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
                        tokens.push(Token {ty: TokenT::Comparator(Comparator::GTE), idx});
                        idx+=2;
                    } else {
                        tokens.push(Token {ty: TokenT::Comparator(Comparator::GT), idx});
                        idx+=1;
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Comparator(Comparator::LTE), idx});
                        idx+=2;
                    } else {
                        tokens.push(Token {ty: TokenT::Comparator(Comparator::LT), idx});
                        idx+=1;
                    }
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Comparator(Comparator::EQ), idx});
                        idx+=2;
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token {ty: TokenT::Comparator(Comparator::NEQ), idx});
                        idx+=2;
                    }
                }
                '&' => {
                    tokens.push(Token {ty: TokenT::ConditionalOperator(ConditionalOperator::AND), idx});
                    chars.next();
                    idx+=1;
                }
                '|' => {
                    tokens.push(Token {ty: TokenT::ConditionalOperator(ConditionalOperator::OR), idx});
                    chars.next();
                    idx+=1;
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
        println!("{:#?}", tokens);
        tokens
    }

    fn parse_condition<I>(
        iter: &mut Peekable<I>,
    ) -> Result<ASTNode, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        match iter.peek() {
            Some(Token{ ty: TokenT::ConditionalOperator(_), ..}) => Self::parse_logical_op(iter),
            Some(Token{ ty: TokenT::Literal(_), idx}) => {
                let idx_clone = idx.clone();
                if let Some(Token{ ty: TokenT::Literal(literal), ..}) = iter.next() {
                    Ok(ASTNode::Literal(literal))
                } else {
                    println!("here1");
                    Err(ParseError{ ty: ParseErrorT::Unexpected, cursor: idx_clone})
                }
            }
            Some(Token{ ty: TokenT::Number(_), idx}) => {
                let idx_clone = idx.clone();
                if let Some(Token{ ty: TokenT::Number(num), ..}) = iter.next() {
                    Ok(ASTNode::Number(num))
                } else {
                    Err(ParseError{ ty: ParseErrorT::Unexpected, cursor: idx_clone})
                }
            }
            Some(Token{ ty: TokenT::OpenParen, idx}) => {
                iter.next();
                let left = Self::parse_condition(iter)?;
                let op = match iter.next() {
                    Some(Token{ ty: TokenT::Comparator(cmp), ..}) => cmp,
                    _ => return Err(ParseError{ ty: ParseErrorT::MissingComparator, cursor: 0/*TODO*/}),
                };
                let right = Self::parse_condition(iter)?;
    
                match iter.next() {
                    Some(Token { ty: TokenT::CloseParen, ..}) => Ok(ASTNode::Condition {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    }),
                    _ => {
                        println!("here3");
                        return Err(ParseError{ ty: ParseErrorT::UnmatchedParenthesis, cursor: 0 /*TODO!!!*/});
                    }
                }
            }
    
            _ => {
                println!("here6");
                return Err(ParseError{ ty: ParseErrorT::Unexpected, cursor: 0 /*TODO!!!*/});
            }
        }
    }
    
    fn parse_logical_op<I>(
        iter: &mut Peekable<I>,
    ) -> Result<ASTNode, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        let op = match iter.next() {
            Some(Token {ty: TokenT::ConditionalOperator(cond_op), idx}) => cond_op,
            _ => panic!("expected conditional operator")
        };
    
        match iter.next() {
            Some(Token {ty: TokenT::OpenParen, idx}) => {}
            _ => return Err(ParseError {ty: ParseErrorT::MissingOpenParen, cursor: 0/*TODO*/}),
        }
        let mut conditions = Vec::new();

        loop {
            let condition = Self::parse_condition(iter)?;
            conditions.push(Box::new(condition));
            println!("{:?}", conditions);
            match iter.peek() {
                Some(Token{ ty: TokenT::CloseParen, ..}) => {
                    // iter.next();
                    break;
                }
                Some(Token{ ty: TokenT::OpenParen, ..}) => {
                    continue;
                }
                _ => {
                    println!("here5");
                    return Err(ParseError {ty: ParseErrorT::Unexpected, cursor: 0 /*TODO:handle index of this properly*/});
                }
            }
        }
        Ok(ASTNode::ConditionalOperator {
            op,
            conditions,
        })
    }    

    fn parse_match<I>(
        iter: &mut Peekable<I>,
    ) -> Result<ASTNode, ParseError>
    where
        I: Iterator<Item = Token>,
    {
        match iter.next() {
            Some(Token{ ty: TokenT::Match, idx}) => {
                match iter.next() {
                    Some(Token{ ty: TokenT::OpenParen, ..}) => {}
                    _ => return Err(ParseError{ ty: ParseErrorT::MissingOpenParen, cursor: idx}),
                }
    
                let condition_chain = Self::parse_condition(iter)?;
                println!("{:?}", condition_chain);
                match iter.peek() {
                    Some(Token{ ty: TokenT::CloseParen, idx}) => {
                        iter.next();
                        Ok(ASTNode::Match(Box::new(condition_chain)))
                    }
                    _ => {
                        println!("here4");
                        return Err(ParseError{ ty: ParseErrorT::UnmatchedParenthesis, cursor: 0/*TODO*/});
                    }
                }
            }
            _ => {
                println!("here2");
                return Err(ParseError{ ty: ParseErrorT::Unexpected, cursor: 0/*0*/});
            }
        }
    }
    

    pub fn parse_tokens(&mut self, tokens: &Vec<Token>) -> Result<(), ParseError>{
        let mut nodes = Vec::new();
        let mut iter = tokens.iter().cloned().peekable();
        while let Some(t) = iter.peek() {
            match t.ty {
                TokenT::Match => {
                    match Self::parse_match(&mut iter){
                        Ok(node) => nodes.push(node),
                        Err(e) => return Err(e)
                    };
                }
                _ => return Err(ParseError {ty: ParseErrorT::Unexpected, cursor: t.idx}),
            }
        }
        self.ast = nodes;
        Ok(())
    }

    pub fn ast2mql(&self) -> String {
        let mut s = String::from("db.collection.aggregate{[");
        for node in self.ast.iter() {
            if let ASTNode::Match(inner) = node {
                if let ASTNode::Condition { op, left, right } = &**inner {
                    if let (ASTNode::Literal(left), ASTNode::Literal(right)) = (&**left, &**right) {
                        let op_str = match op {
                            Comparator::GTE => "$gte",
                            Comparator::GT => "$gt",
                            Comparator::EQ => "$eq",
                            Comparator::NEQ => "$neq",
                            Comparator::LT => "$lt",
                            Comparator::LTE => "$lte",
                        };
                        s.push_str(&format!(
                            "{{ $match: {{ {}: {{ {}: {} }} }} }},",
                            left, op_str, right
                        ));
                    }
                }
            } else {
                panic!("Unexpected node type!");
            }
        }
        s.push_str("]}");
        s
    }
}