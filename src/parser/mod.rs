    
use crate::tokens;
use std::collections::HashMap;
use std::str::Chars;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::io::prelude::*;


pub enum InstructionResult {
    EvaluateNext,
    EvaluateLine(i16),
    EvaluateEnd
}

pub trait GwInstruction {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult;
    fn fill_structure_string(&self,   buffer : &mut String);
}

#[derive(Clone)]
pub enum ExpressionEvalResult {
    StringResult(String),
    IntegerResult(i16)    
}

impl ExpressionEvalResult {
    fn to_string(&self) -> String {
        match self {
            ExpressionEvalResult::StringResult(some_string) => some_string.clone(),
            ExpressionEvalResult::IntegerResult(iresult) => String::from(iresult.to_string())
        }
    }
}

pub struct EvaluationContext {
    variables: HashMap<String, ExpressionEvalResult>,
    jump_table: HashMap<i16, i16>
}

impl EvaluationContext {
    fn get_real_line(&self, referenced_line : i16) -> Option<i16> {
        if let Some(lin) =  self.jump_table.get(&referenced_line) {
            Some(*lin)
        } else {
            None
        }
    }
    
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
        match (left_result, right_result) {
            (ExpressionEvalResult::IntegerResult(left),
             ExpressionEvalResult::IntegerResult(right)) =>
                ExpressionEvalResult::IntegerResult(left + right),
            (_, _) => panic!("Not implemented")
            
        }
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
    line : i16,
    instruction : Box<dyn GwInstruction>,
    rest_instructions : Option<Vec<Box<dyn GwInstruction>>>
}

impl ProgramLine {
    fn get_line(&self) -> i16 {
        self.line
    }
    
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult {
         self.instruction.eval(context)
    }

    fn fill_structure_string(&self,   buffer : &mut String) {
        buffer.push('(');
        buffer.push_str(&self.line.to_string()[..]);
        buffer.push(' ');
        self.instruction.fill_structure_string(buffer);
        if let Some(rest) = &self.rest_instructions {
            buffer.push(' ');
            for e in rest {
                buffer.push(':');
                e.fill_structure_string(buffer);
            }
        }
        buffer.push(')');        
    }
}


pub struct GwCls {
    
}

impl GwInstruction for GwCls {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"CLS");
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

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.variable[..]);
        buffer.push_str(&" = ");
        self.expression.fill_structure_string(buffer);
    }    
}

struct GwGotoStat {
    line : i16
}

impl GwInstruction for GwGotoStat {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        if let Some(actual_line) =  context.get_real_line(self.line) {
                 return InstructionResult::EvaluateLine(actual_line);
        } else {
            println!("-- {}", self.line);
            panic!("Trying to jump to non existing line");
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("GOTO ");
    }
}

enum SwitchIndicator {
    On, Off
}

struct GwKeyStat {
    indicator : SwitchIndicator
}

impl GwInstruction for GwKeyStat {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
       InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"KEY ...");
    }
}

struct GwColor {
    red : Box<dyn GwExpression>,
    green : Box<dyn GwExpression>,
    blue : Box<dyn GwExpression>    
}

impl GwInstruction for GwColor {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"COLOR ");
        self.red.fill_structure_string(buffer);
        buffer.push_str(&", ");
        self.green.fill_structure_string(buffer);
        buffer.push_str(&", ");
        self.blue.fill_structure_string(buffer);
    }
}

struct GwPrintStat {
    expression : Box<dyn GwExpression>
}

impl GwInstruction for GwPrintStat {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        let result = self.expression.eval(context);
        println!("{}", result.to_string());
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"PRINT ");
        self.expression.fill_structure_string(buffer);
    }
}

struct GwInputStat {
    variable : String
}

impl GwInstruction for GwInputStat {
    fn eval(&self, context : &mut EvaluationContext) ->
        InstructionResult {
            
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"INPUT ");
        buffer.push_str(&self.variable[..]);
    }
}


pub struct GwProgram {
    lines : Vec<ProgramLine>,
//    lines_index : Vec<u16>
}

impl GwProgram {
    pub fn new() -> GwProgram {
        GwProgram {
            lines: Vec::new(),
        }
    }

    pub fn load_from(&mut self, file_name : &str) -> io::Result<()> {
        let f = File::open(file_name)?;
        let mut reader = BufReader::new(f);
        let mut line_number = 1;
        for line in reader.lines() {
            let uline = line.unwrap();
            println!(">> {}", uline);
            match parse_instruction_line_from_string(uline) {
               ParserResult::Success(parsed_line) => self.add_line(parsed_line),
                ParserResult::Error(error) => {println!("Line {} Error: {}", line_number, error); break;},
               ParserResult::Nothing=> println!("Nothing")       
            }
            line_number = line_number + 1;
        }
        Ok(())
    }

    pub fn list(&self) {
        for element in self.lines.iter() {
            let mut string_to_print = String::new();
            element.fill_structure_string(&mut string_to_print);
            println!("{}", string_to_print);
        }
    }

    pub fn run(&self) {
        let mut table = HashMap::new();
        let mut i = 0;
        for e in self.lines.iter() {
            table.insert(e.get_line(), i);
            i = i + 1;
        }
        let mut context = EvaluationContext {
            variables: HashMap::new(),
            jump_table: table
        };
        self.eval(&mut context);
    }
    
    pub fn add_line(&mut self, new_line : ProgramLine) {
        let mut i = 0;        
        while i < self.lines.len() {
            if new_line.get_line() == self.lines[i].get_line() {
                self.lines[i] = new_line;
                return;
            } else if new_line.get_line() < self.lines[i].get_line() {
                self.lines.insert(i, new_line);
                return;
            }
            i = i + 1;
        }
        self.lines.push(new_line);
    }
    
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
                    current_index = usize::try_from(new_line).unwrap();
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

#[derive(Debug)]
#[derive(Clone)]
pub enum GwToken {
    Keyword(tokens::GwBasicToken),
    Identifier(String),
    String(String),
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


// fn recognize_operator<'a>(iterator : &mut PushbackCharsIterator<'a>)
//                           -> ParserResult<GwBinaryOperationKind> {
//     match iterator.next() {
//         Some(the_char) => {
//             if the_char.is_alphabetic() {
//                 iterator.push_back(the_char);
//                 if let Some(operator_name) = recognize_word(iterator) {
//                     match operator_name[..] {
//                         _ => return  ParserResult::Error(String::from("Error expecting AND, OR, XOR, IMP, etc"))
//                     } 
//                 } else {
//                     return  ParserResult::Error(String::from("Error expecting alphabetic operator"));
//                 }
//             } else {
//                 match the_char {
//                     '+' => ParserResult::Success(GwBinaryOperationKind::Plus),
//                     '-' => ParserResult::Success(GwBinaryOperationKind::Minus),
//                     _ => ParserResult::Nothing    
//                 }                    
//             }
//         }
//         None => ParserResult::Nothing
//     }
// }

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
            // Controversial! but it seems that GwBasic
            // changes the case of identifers
            let mut upper_case_word = word.clone();
            upper_case_word.make_ascii_uppercase();

            if let Some(kw) =  self.tokens_info.get_token(&upper_case_word) {
                return Some(GwToken::Keyword(kw.clone()))
            } else {
                return Some(GwToken::Identifier(upper_case_word));
            }
        } else if let Some(int_number) = recognize_int_number_str(&mut self.chars_iterator) {
            return Some(GwToken::Integer(int_number));
        } else if let Some(string_literal) = recognize_string_literal(&mut self.chars_iterator) {
            return Some(GwToken::String(string_literal));
        } else if recognize_specific_char(&mut self.chars_iterator, '+') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::PlusTok));
        } else if recognize_specific_char(&mut self.chars_iterator, '=') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::EqlTok));            
            
        } else if recognize_specific_char(&mut self.chars_iterator, '*') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::TimesTok));
        } else if recognize_specific_char(&mut self.chars_iterator, ':') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::ColonSeparatorTok));
        } else if recognize_specific_char(&mut self.chars_iterator, ',') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::CommaSeparatorTok));            
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
        } else if let GwToken::String(str_val) = next_token {
            ParserResult::Success(Box::new(GwStringLiteral { value: str_val}))                                            
        } else {
            ParserResult::Error(String::from(format!("Unexpected token parsing single expression:  {:?}", next_token)))
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
             iterator.push_back(next_token);
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

fn parse_print_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    if let ParserResult::Success(expr) = parse_expression(iterator) {
        return ParserResult::Success(Box::new(
            GwPrintStat {
                expression: expr
            }
        ));
    } else {
        return ParserResult::Error(String::from("Expecting expression as PRINT argument"));
    }
}

fn parse_input_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    if let Some(GwToken::Identifier(id)) = iterator.next() {
        return ParserResult::Success(Box::new(
            GwInputStat {
                variable: id
            }
        ));
    } else {
        return ParserResult::Error(String::from("Expecting variable as INPUT argument"));
    }
}

fn parse_cls_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(
        GwCls {}
        ));
}

fn parse_key_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    let next_token = iterator.next();
    match next_token {
        Some(GwToken::Keyword(tokens::GwBasicToken::OnTok)) =>
            ParserResult::Success(Box::new(GwKeyStat { indicator : SwitchIndicator::On })),
        Some(GwToken::Keyword(tokens::GwBasicToken::OffTok)) =>
            ParserResult::Success(Box::new(GwKeyStat { indicator : SwitchIndicator::Off })),
        _ => ParserResult::Error(String::from("Error parsing Key statement"))
    }
}


fn parse_with_separator<'a, F,T : ?Sized>(
    iterator : &mut PushbackTokensIterator<'a>,
    item_parse_fn  : F,
    separator : tokens::GwBasicToken) ->
     ParserResult<Vec<Box< T>>>
  where F : Fn(&mut PushbackTokensIterator<'a>) -> ParserResult<Box< T>>  {
    let mut result = Vec::new();
    loop {
        let item_result = item_parse_fn(iterator);
        if let ParserResult::Success(item) = item_result {
            result.push(item);
        } else {
            return ParserResult::Error(String::from("error parsing sequence"));
        }
        let next_token = iterator.next();
        match next_token {
            Some(GwToken::Keyword(kw_tok)) if separator == kw_tok => continue,
            Some(token_result) => {
                iterator.push_back(token_result);
                break;
            },
            _ => break
        }
    }
    return ParserResult::Success(result);
}

fn parse_color_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    let color_components = parse_with_separator(
        iterator,
        parse_expression,
        tokens::GwBasicToken::CommaSeparatorTok);
    if let ParserResult::Success(components) = color_components {
        let mut mut_components = components;
        if mut_components.len() == 3 {
            let mut drain_iterator = mut_components.drain(0..);
            let red_expr = drain_iterator.next().unwrap();
            let green_expr = drain_iterator.next().unwrap();
            let blue_expr = drain_iterator.next().unwrap();
            
            return ParserResult::Success(Box::new(GwColor {
                red: red_expr,
                green: green_expr,
                blue: blue_expr
            }));
        } else {
           return ParserResult::Error(String::from("Error parsing Color color components"));            
        }
    } else {
        return ParserResult::Error(String::from("Error recognizing colors"));
    }    
}

                      

fn parse_goto_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    if let Some(GwToken::Integer(line)) = iterator.next() {
        return ParserResult::Success(Box::new(
            GwGotoStat {
                line: line
            }
        ));
    } else {
        return ParserResult::Error(String::from("Expecting number at GOTO statement"));
    }
}


fn parse_assignment<'a>(iterator : &mut PushbackTokensIterator<'a>, identifier : String)
                        -> ParserResult<Box<dyn GwInstruction>> {

    if let Some(next_token)  = iterator.next() {
        if let GwToken::Keyword(tokens::GwBasicToken::EqlTok) = next_token {
            if let ParserResult::Success(expr) =  parse_expression(iterator) {
                return ParserResult::Success(
                    Box::new(
                    GwAssign {
                        variable: identifier,
                        expression: expr
                    }));
            } else {
                return ParserResult::Error(String::from("Error parsing  assignment"));
            }
        } else {
            return ParserResult::Error(String::from("Expecting assignment"));
        }
    } else {
        return ParserResult::Error(String::from("cannot parse assignment"));
    }
}

fn parse_instruction<'a>(iterator : &mut PushbackTokensIterator<'a>) -> ParserResult<Box<dyn GwInstruction>> {
    if let Some(next_tok) = iterator.next() {
        match next_tok {
            GwToken::Keyword(tokens::GwBasicToken::GotoTok) => parse_goto_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::ClsTok) => parse_cls_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::ColorTok) => parse_color_stat(iterator),                        
            GwToken::Keyword(tokens::GwBasicToken::KeyTok) => parse_key_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::PrintTok)  => parse_print_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::InpTok)  => parse_input_stat(iterator),            
            GwToken::Identifier(var_name) => parse_assignment(iterator, var_name),
            _ => panic!("Not implemented parsing for non-assigment")
        }
        /*
        if let GwToken::Keyword(tokens::GwBasicToken::GotoTok) = next_tok {
            return parse_goto_stat(iterator);
        }        
        else if let GwToken::Keyword(tokens::GwBasicToken::PrintTok) = next_tok {
            return parse_print_stat(iterator);
        }
        else if let GwToken::Identifier(var_name) = next_tok {
            return parse_assignment(iterator, var_name);
        } else {
            panic!("Not implemented parsing for non-assigment");
        }*/
    } else {
        return ParserResult::Nothing;
    }    
}

pub fn parse_instruction_line_from_string(line : String)
                                          -> ParserResult<ProgramLine> {
    
    let mut pb = PushbackCharsIterator {
        chars: line.chars(),
        pushed_back: None
    };
    let mut tokens_iterator = PushbackTokensIterator::create(pb);
    parse_instruction_line(&mut tokens_iterator)
}

fn parse_same_line_instruction_sequence<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                            -> ParserResult<Vec<Box<dyn GwInstruction>>> {
    if let Some(next_tok) = iterator.next() {
                            println!(">>>>> {:?}", next_tok);
        if let GwToken::Keyword(tokens::GwBasicToken::ColonSeparatorTok) = next_tok {
            iterator.push_back(next_tok);
            return parsing_same_line_instruction_sequence(iterator)
        } else {
            return ParserResult::Error(String::from("Expecting colon")); 
        }
    } else {
        return ParserResult::Nothing;
    }
} 

fn parsing_same_line_instruction_sequence<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                              -> ParserResult<Vec<Box<dyn GwInstruction>>> {
    let mut results = Vec::<Box<dyn GwInstruction>>::new();    
    while let Some(next_tok) = iterator.next() {
        if let GwToken::Keyword(tokens::GwBasicToken::ColonSeparatorTok) = next_tok {
            let instr_result = parse_instruction(iterator);
            if let ParserResult::Success(parsed_instruction) = instr_result {
                results.push(parsed_instruction);
            } else {
                return ParserResult::Error(String::from("Error parsing same line statement"));
            }
        } else {
            return ParserResult::Error(String::from("Expecting colon"));
        }
    } 
    return ParserResult::Success(results);
} 

pub fn parse_instruction_line<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                  -> ParserResult<ProgramLine> {
    if let Some(next_tok) = iterator.next() {
        if let GwToken::Integer(line_number) = next_tok {
            if let ParserResult::Success(instr) = parse_instruction(iterator) {
                let rest_parsing_result = parse_same_line_instruction_sequence(iterator);
                if let ParserResult::Success(rest_inst) = rest_parsing_result {
                    return ParserResult::Success(
                        ProgramLine {
                            line : line_number,
                            instruction : instr,
                            rest_instructions : Some(rest_inst)
                        }
                    );
                } else if let ParserResult::Nothing = rest_parsing_result {                    
                    return ParserResult::Success(
                        ProgramLine {
                            line : line_number,
                            instruction : instr,
                            rest_instructions : None
                        }
                    );
                } else {
                    return  ParserResult::Error(String::from("Error parsing line (rest)"));

                }
            } else {
                ParserResult::Error(String::from("Error parsing line"))
            }
        } else {
            ParserResult::Error(String::from("line number expected"))
        }
    } else {
        ParserResult::Nothing
    }
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
            variables: HashMap::new(),
            jump_table: HashMap::new()
                
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
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(AB + BC)"));
            }
            _ => panic!("errror")
        }        
    }

    #[test]
    fn it_parses_assignment_instruction() {
        let str = "10 x = ab";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_instruction_line(&mut tokens_iterator) {
            ParserResult::Success(instr) => {
                let mut buf = String::new();
                instr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(10 X = AB)"));
            }
            _ => panic!("errror")
        }        
    }

    #[test]
    fn it_parses_mult_instruction_line() {
        let str = "10 x = ab : y = bc : z = cd";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_instruction_line(&mut tokens_iterator) {
            ParserResult::Success(instr) => {
                let mut buf = String::new();
                instr.fill_structure_string(&mut buf);

                assert_eq!(buf, String::from("(10 X = AB :Y = BC:Z = CD)"));
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
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("((AB + BC) + CD)"));
            }
            _ => panic!("errror")
        }        
    }

    #[test]
    fn it_parses_a_sequence_with_separator() {
        let str = "ab , 12 , a + cd";
        let mut pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_with_separator(&mut tokens_iterator,
                                   parse_expression,
                                   tokens::GwBasicToken::CommaSeparatorTok) {
            ParserResult::Success(vect) => {
                assert_eq!(3, vect.len());
                let mut buf = String::new();
                vect[0].fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("AB"));

                let mut buf = String::new();
                vect[1].fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("12"));
                
                let mut buf = String::new();
                vect[2].fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(A + CD)"));               
            }
            _ => panic!("errror")
        }        
    }


    #[test]
    fn it_parses_color() {
        let str = "color 1,  2,3";
        let  pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_instruction(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("COLOR 1, 2, 3"));
            },
            ParserResult::Error(err) => {
                panic!("Error parsing: {}", err);
            },
            _ => panic!("errror")
        }        
    }     
    
    

    #[test]
    fn it_parses_multiplicative_four() {
        let str = "ab + bc + cd *  de";
        let  pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("((AB + BC) + (CD * DE))"));
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
