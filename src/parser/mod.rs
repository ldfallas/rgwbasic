
use crate::tokens;
use std::collections::HashMap;
use std::str::Chars;

pub enum InstructionResult {
    EvaluateNext,
    EvaluateLine(u16),
    EvaluateEnd
}

pub trait GwInstruction {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult;
}

#[derive(Clone)]
pub enum ExpressionEvalResult {
    StringResult(String),
    IntegerResult(i16)    
}

pub struct EvaluationContext {
    variables: HashMap<String, ExpressionEvalResult>
}

impl EvaluationContext {
    fn lookup_variable(&self, name : &String) -> Option<&ExpressionEvalResult> {
        self.variables.get(name)
    }
    
    fn set_variable(&mut self, name : &String, value : &ExpressionEvalResult) {
        self.variables.insert(name.clone(), value.clone());
    }
}

pub trait GwExpression {
    fn eval(&self, context : &mut EvaluationContext) -> ExpressionEvalResult ;
    fn fill_structure_string(&self,   buffer : &mut String);
}

pub struct GwStringLiteral {
    value : String
}

impl GwExpression for GwStringLiteral {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::StringResult(String::from(&self.value))
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.value[..]);
    }
}

pub struct GwIntegerLiteral {
    value : i16
}

impl GwExpression for GwIntegerLiteral {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::IntegerResult(self.value)
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.value.to_string());
    }    
}


pub struct GwVariableExpression {
    name : String
}

impl GwExpression for GwVariableExpression {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        if let Some(value) =  context.lookup_variable(&self.name) {
            value.clone()
        } else {
            // TODO we need to define a variable here???
            ExpressionEvalResult::IntegerResult(0)
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.name[..]);
    }
}


pub enum GwBinaryOperationKind {
    Plus,
    Minus,
    Times,
    FloatDiv,
    IntDiv,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
    Equal,
    Different,
    Exponent,
    Mod,             
    And,
    Or,
    Eqv,
    Xor,
    Implication   
}


pub struct GwBinaryOperation {
    kind: GwBinaryOperationKind,
    left: Box<dyn GwExpression>,
    right: Box<dyn GwExpression>
}

impl GwBinaryOperation {
   fn fill_operator(&self, buffer : &mut String) {
        match self.kind {
            GwBinaryOperationKind::Plus => buffer.push_str(" + "),
            GwBinaryOperationKind::Times => buffer.push_str(" * "),
            _ => buffer.push_str(" ?? ")
        }
    }
}

impl GwExpression for GwBinaryOperation {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        panic!("Not implemented");
    }
    fn fill_structure_string(&self,   val : &mut String) {
        val.push_str("(");
        self.left.fill_structure_string(val);
        self.fill_operator(val);
        self.right.fill_structure_string(val);        
        val.push_str(")");
    }
}
 


pub struct ProgramLine {
    line : u16,
    instruction : Box<dyn GwInstruction>,
    rest_instructions : Option<Box<dyn GwInstruction>>
}

impl ProgramLine {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult {
         self.instruction.eval(context)
    }
}


pub struct GwCls {
    
}

impl GwInstruction for GwCls {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        InstructionResult::EvaluateNext
    }
}

pub struct GwAssign {
    variable : String,
    expression : Box<dyn GwExpression>
}

impl GwInstruction for GwAssign {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        let expression_evaluation = self.expression.eval(context);
        context.set_variable(&self.variable, &expression_evaluation);
        InstructionResult::EvaluateNext
    }
}

pub struct GwProgram {
    lines : Vec<ProgramLine>,
//    lines_index : Vec<u16>
}

impl GwProgram {
    fn eval(&self, context : &mut EvaluationContext) {
        let mut current_index = 0;
        loop {
            if current_index >= self.lines.len() {
                break;
            }

            let eval_result = self.lines[current_index].eval(context);
            match eval_result {
                InstructionResult::EvaluateNext => {
                    current_index = current_index + 1;
                }
                InstructionResult::EvaluateLine(new_line) => {
                    panic!("aaaaaahhhh");
                }
                InstructionResult::EvaluateEnd => {
                    break;
                }
            }            
        }
    }
}



pub struct PushbackCharsIterator<'a> {
    chars : Chars<'a>,
    pushed_back: Option<char>
}
impl PushbackCharsIterator<'_> {
    fn next(&mut self) -> Option<char> {
        if let Some(pushed) = self.pushed_back {
            self.pushed_back = None;
            Some(pushed)
        } else {
            self.chars.next()
        }
    }

    fn push_back(&mut self, char_to_push : char) {
        self.pushed_back = Some(char_to_push);
    }
}

#[derive(Clone)]
pub enum GwToken {
    Keyword(tokens::GwBasicToken),
    Identifier(String),
    StringTok(String),
    Integer(i16),
    Comma,
    Colon
}


fn consume_whitespace<'a>(iterator : &mut PushbackCharsIterator<'a>) {
    loop {
        if let Some(c) = iterator.next() {
            if c != ' ' {
                iterator.push_back(c);
                break;
            }
        } else {
            break;
        }
    }
}

fn recognize_specific_char<'a>(iterator : &mut PushbackCharsIterator<'a>, c : char) -> bool {
    if let Some(next_char) = iterator.next()  {
        let result = (next_char == c);
        if !result {
           iterator.push_back(next_char);
        }
        result
    } else {
        false 
    }    
}

fn recognize_eol<'a>(iterator : &mut PushbackCharsIterator<'a>) -> bool {
    if let Some(next_char) = iterator.next()  {
        iterator.push_back(next_char);
        false
    } else {
        true
    }    
}


fn recognize_operator<'a>(iterator : &mut PushbackCharsIterator<'a>)
                          -> ParserResult<GwBinaryOperationKind> {
    match iterator.next() {
        Some(the_char) => {
            if the_char.is_alphabetic() {
                iterator.push_back(the_char);
                if let Some(operator_name) = recognize_word(iterator) {
                    match operator_name[..] {
                        _ => return  ParserResult::Error(String::from("Error expecting AND, OR, XOR, IMP, etc"))
                    } 
                } else {
                    return  ParserResult::Error(String::from("Error expecting alphabetic operator"));
                }
            } else {
                match the_char {
                    '+' => ParserResult::Success(GwBinaryOperationKind::Plus),
                    '-' => ParserResult::Success(GwBinaryOperationKind::Minus),
                    _ => ParserResult::Nothing    
                }                    
            }
        }
        None => ParserResult::Nothing
    }
}

fn recognize_word<'a>(iterator : &mut PushbackCharsIterator<'a>) -> Option<String> {
    if let Some(next_char) = iterator.next() {
        if next_char.is_alphabetic() {
            let mut result = String::new();
            result.push(next_char);
            loop {
                if let Some(next_char) = iterator.next() {
                    if next_char.is_alphabetic()
                        || next_char.is_digit(10) {
                       result.push(next_char);
                    } else {
                        iterator.push_back(next_char);
                        return Some(result);
                    }
                } else {
                    return Some(result);
                }
            }
        } else {
            iterator.push_back(next_char);
        }
    }
    None
}

fn recognize_string_literal<'a>(iterator : &mut PushbackCharsIterator<'a>) -> Option<String> {
    if let Some(next_char) = iterator.next() {
        if next_char == '"' {
            let mut result = String::new();
            result.push(next_char);
            loop {
                if let Some(next_char) = iterator.next() {
                    result.push(next_char);
                    if next_char == '"' {
                       return Some(result);
                    }
                } else {
                    return Some(result);
                }
            }
        } else {
            iterator.push_back(next_char);
        }
    }
    None
}


fn recognize_int_number_str<'a>(iterator : &mut PushbackCharsIterator<'a>) -> Option<i16> {
    if let Some(c) = iterator.next()  {
        if c.is_digit(10) {
            let mut tmp_string = String::new();
            tmp_string.push(c);
            loop {
                if let Some(c) = iterator.next() {
                    if c.is_digit(10) {
                        tmp_string.push(c);                        
                    } else {
                        iterator.push_back(c);
                        break;
                    }
                } else {
                    break;
                }
            }
            if let Ok(result) = i16::from_str_radix(&tmp_string.to_string(), 10) {
                Some(result)
            } else {
                None
            }
        } else {
            iterator.push_back(c);
            None
        }
    } else {
        None
    }
}


pub struct PushbackTokensIterator<'a> {
    chars_iterator: PushbackCharsIterator<'a>,
    tokens_info: tokens::GwTokenInfo,
    pushed_back: Option<GwToken>
}

impl<'a> PushbackTokensIterator<'a> {
    fn create(chars_iterator : PushbackCharsIterator<'a>) -> PushbackTokensIterator<'a> {
        PushbackTokensIterator {
            chars_iterator: chars_iterator,
            tokens_info: tokens::GwTokenInfo::create(),
            pushed_back: None
        }
    }
    
    fn next(&mut self) -> Option<GwToken> {

        // TODO interesting how to accomplish the
        // following code without clonning the token
        if let Some(pushed) = &self.pushed_back {
            let copy = pushed.clone();
            self.pushed_back = None;
            return Some(copy);
        }
        
        consume_whitespace(&mut self.chars_iterator);
        if let Some(word) = recognize_word(&mut self.chars_iterator) {
            if let Some(kw) =  self.tokens_info.get_token(&word) {
                return Some(GwToken::Keyword(kw.clone()))
            } else {
                return Some(GwToken::Identifier(word));
            }
        } else if let Some(int_number) = recognize_int_number_str(&mut self.chars_iterator) {
            return Some(GwToken::Integer(int_number));
        } else if recognize_specific_char(&mut self.chars_iterator, '+') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::PlusTok));
        } else if recognize_specific_char(&mut self.chars_iterator, '*') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::TimesTok));            
        } else if recognize_eol(&mut self.chars_iterator) {
            return None;
        }
        panic!("Not implemented scenario of non recognized token!");
    }

    pub fn push_back(&mut self, tok_to_push : GwToken) {
        self.pushed_back = Some(tok_to_push);
    }
}


pub enum ParserResult<T> {
    Success(T),
    Error(String),
    Nothing
}

pub fn parse_single_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                               -> ParserResult<Box<dyn GwExpression>> {
    if let Some(next_token) = iterator.next() {
        if let GwToken::Identifier(id) = next_token {
            ParserResult::Success(Box::new(GwVariableExpression { name: id}))
        } else if let GwToken::Integer(i_val) = next_token {
            ParserResult::Success(Box::new(GwIntegerLiteral { value: i_val}))                            
        } else {
            ParserResult::Error(String::from("??"))
        }
    } else {
        ParserResult::Nothing
    }
}



fn one_kw_token_of<'a>(token : &'a GwToken, t1 : &'a tokens::GwBasicToken, t2 : &'a tokens::GwBasicToken) -> Option<&'a tokens::GwBasicToken>{
    match token  {
        GwToken::Keyword(tok) if *tok == *t1 || *tok == *t2 => Some(tok),
        _ => None
    }        
}

////
pub fn parse_multiplicative_expressions<'a>(iterator : &mut PushbackTokensIterator<'a>, current : Box<dyn GwExpression>)
                                          -> ParserResult<Box<dyn GwExpression>> {

  let mut current_expr = current;
  loop {
     if let Some(next_token) = iterator.next() {
         if let Some(tok) = one_kw_token_of(&next_token, &tokens::GwBasicToken::DivTok, &tokens::GwBasicToken::TimesTok) {
             if let ParserResult::Success(right_side_parse_result) = parse_multiplicative_expression(iterator) {

                current_expr = 
                         Box::new(
                            GwBinaryOperation {
                                kind: GwBinaryOperationKind::Times,
                                left: current_expr,
                                right: right_side_parse_result });
             } else {
                 return ParserResult::Error(String::from("Error parsing additive expression, expecting right side operand "));
             }
         } else {
             iterator.push_back(next_token);
             return ParserResult::Success(current_expr);
         }
     } else {
         return ParserResult::Success(current_expr);
     }
  }
}



pub fn parse_multiplicative_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                     -> ParserResult<Box<dyn GwExpression>> {
    match parse_single_expression(iterator) {
        ParserResult::Success(left_side_parse_result) => {
            return parse_multiplicative_expressions(iterator, left_side_parse_result);
        },
        ParserResult::Nothing => {
            ParserResult::Nothing
        }        
        ParserResult::Error(msg) => {
            ParserResult::Error(msg)
        }
    }
}

////


pub fn parse_additive_expressions<'a>(iterator : &mut PushbackTokensIterator<'a>, current : Box<dyn GwExpression>)
                                          -> ParserResult<Box<dyn GwExpression>> {

  let mut current_expr = current;
  loop {
     if let Some(next_token) = iterator.next() {
         if let Some(tok) = one_kw_token_of(&next_token, &tokens::GwBasicToken::PlusTok, &tokens::GwBasicToken::MinusTok) {
             if let ParserResult::Success(right_side_parse_result) = parse_multiplicative_expression(iterator) {

                current_expr = 
                         Box::new(
                            GwBinaryOperation {
                                kind: GwBinaryOperationKind::Plus,
                                left: current_expr,
                                right: right_side_parse_result });
             } else {
                 return ParserResult::Error(String::from("Error parsing additive expression, expecting right side operand "));
             }
         } else {
             return ParserResult::Success(current_expr);
         }
     } else {
         return ParserResult::Success(current_expr);
     }
  }
}



pub fn parse_additive_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                     -> ParserResult<Box<dyn GwExpression>> {

    match parse_multiplicative_expression(iterator) {
        ParserResult::Success(left_side_parse_result) => {
            return parse_additive_expressions(iterator, left_side_parse_result);
        },
        ParserResult::Nothing => {
            ParserResult::Nothing
        }        
        ParserResult::Error(msg) => {
            ParserResult::Error(msg)
        }
    }
}

pub fn parse_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                            -> ParserResult<Box<dyn GwExpression>> {
    return parse_additive_expression(iterator);
}


#[cfg(test)]
mod eval_tests {
    use std::collections::HashMap;
    
    // use crate::gwparser::GwAssign;
    // use crate::gwparser::GwIntegerLiteral;
    // use crate::gwparser::ProgramLine;
    // use crate::gwparser::GwProgram;
    // use crate::gwparser::EvaluationContext;
    // use crate::gwparser::ExpressionEvalResult;
    use crate::parser::*;

    
    #[test]
    fn it_tests_basic_eval() {
        let line1 = ProgramLine {
            line: 10,
            instruction: Box::new(GwAssign {
                variable: String::from("X"),
                expression: Box::new( GwIntegerLiteral {
                    value: 10
                })
            }),
            rest_instructions: None
        };

        let program  = GwProgram {
            lines: vec![line1]
        };

        let mut context = EvaluationContext {
            variables: HashMap::new()
        };

        context.variables.insert(String::from("X"), ExpressionEvalResult::IntegerResult(5));

        if let Some(ExpressionEvalResult::IntegerResult(value)) = context.lookup_variable(&String::from("X")) {
            let some_value : i16 = 5;
            assert_eq!(&some_value, value);
        }

        program.eval(&mut context);

        if let Some(ExpressionEvalResult::IntegerResult(value)) = context.lookup_variable(&String::from("X")) {
            let some_value : i16 = 10;
            assert_eq!(&some_value, value);
        }
        
    }
}


#[cfg(test)]
mod parser_tests {
    // use crate::gwparser::PushbackCharsIterator;
    // use crate::gwparser::consume_whitespace;
    // use crate::gwparser::recognize_int_number_str;
    // use crate::gwparser::recognize_word;
    use crate::parser::*;


    #[test]
    fn it_pushes_back() {
        let str = "abc";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };

        if let Some(first) = pb.next() {
            assert_eq!('a', first);
        } else {
            assert!(false, "first character")
        }

        if let Some(second) = pb.next() {
            assert_eq!('b', second);
        } else {
            assert!(false, "second character")
        }

        pb.push_back('x');
        
        if let Some(third) = pb.next() {
            assert_eq!('x', third);
        } else {
            assert!(false, "pushed character")
        }

        if let Some(fourth) = pb.next() {
            assert_eq!('c', fourth);
        } else {
            assert!(false, "fourth character")
        }

        match pb.next() {
            Some(_) => assert!(false),
            None => assert!(true)
        }
    }

    #[test]
    fn it_parses_additive() {
        let str = "ab + bc";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator {
            chars_iterator: pb,
            tokens_info: tokens::GwTokenInfo::create(),
            pushed_back: None
        };
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(ab + bc)"));
            }
            _ => panic!("errror")
        }        
    }

    #[test]
    fn it_parses_additive_three() {
        let str = "ab + bc + cd";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator {
            chars_iterator: pb,
            tokens_info: tokens::GwTokenInfo::create(),
            pushed_back: None
        };
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("((ab + bc) + cd)"));
            }
            _ => panic!("errror")
        }        
    }

    #[test]
    fn it_parses_multiplicative_four() {
        let str = "ab + bc + cd *  de";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        // let mut tokens_iterator = PushbackTokensIterator {
        //     chars_iterator: pb,
        //     tokens_info: tokens::GwTokenInfo::create(),
        //     pushed_back: None
        // };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("((ab + bc) + (cd * de))"));
            }
            _ => panic!("errror")
        }        
    }     
    

    #[test]
    fn it_identifies_numbers() {
        let str = "10 20 30";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };

        match (recognize_int_number_str(&mut pb),
               consume_whitespace(&mut pb),
               recognize_int_number_str(&mut pb),
               consume_whitespace(&mut pb),
               recognize_int_number_str(&mut pb)) {
            (Some(10), _, Some(20), _, Some(30)) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn it_identifies_identifiers() {
        let str = "werwe10    Excs    AJLAKJ";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };

        match (recognize_word(&mut pb),
               consume_whitespace(&mut pb),
               recognize_word(&mut pb),
               consume_whitespace(&mut pb),
               recognize_word(&mut pb)) {
            (Some(w1), _, Some(w2), _, Some(w3))
                if w1.eq(&String::from("werwe10"))
                   && w2.eq(&String::from("Excs"))
                   && w3.eq(&String::from("AJLAKJ"))
              => assert!(true),
            _ => assert!(false)
        }
    }

}
