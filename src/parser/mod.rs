    
use crate::eval::GwAssignableExpression;
use crate::tokens;
use std::str::Chars;
use std::str::FromStr;
use crate::eval::binary::GwBinaryOperationKind;
use crate::eval::SwitchIndicator;
use crate::eval::GwExpression;
use crate::eval::GwVariableExpression;
use crate::eval::GwIntegerLiteral;
use crate::eval::GwDoubleLiteral;
use crate::eval::GwStringLiteral;
use crate::eval::binary::GwBinaryOperation;
use crate::eval::GwInstruction;
use crate::eval::GwListStat;
use crate::eval::GwRunStat;
use crate::eval::GwSystemStat;
use crate::eval::GwPrintStat;
use crate::eval::print_using::GwPrintUsingStat;
use crate::eval::GwLoadStat;
use crate::eval::GwInputStat;
use crate::eval::GwCls;
use crate::eval::GwIf;
use crate::eval::GwKeyStat;
use crate::eval::GwColor;
use crate::eval::GwGotoStat;
use crate::eval::GwAssign;
use crate::eval::GwArrayAssign;
use crate::eval::GwCall;
use crate::eval::while_instr::{GwWhile, GwWend};
use crate::eval::for_instr::{GwFor, GwNext};
use crate::eval::{GwLog, GwInt};
use crate::eval::ProgramLine;
use crate::eval::DefVarRange;
use crate::eval::GwDefDbl;
use crate::eval::GwRem;
use crate::eval::GwParenthesizedExpr;
use crate::eval::GwNegExpr;
use crate::eval::PrintSeparator;


pub struct PushbackCharsIterator<'a> {
    chars : Chars<'a>,
    pushed_back: Option<char>
}

impl PushbackCharsIterator<'_> {
    #[cfg(test)]
    fn new(chars: Chars) -> PushbackCharsIterator {
    	PushbackCharsIterator {
    	    chars,
    	    pushed_back: None
    	}
    }
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
    Double(f32),    
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
        let result = next_char == c;
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

#[cfg(test)]
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


///
///  cases for single 45.32, -1.09E-03,22.5!
///  cases for double 34234234, -1.03432D-06, 342342.0#
///
fn convert_numeric_string(tmp_string : &String)  ->  Option<GwToken> {
    if tmp_string.contains(".")  {
        if let Ok(result) = f32::from_str(&tmp_string.to_string()) {
            Some(GwToken::Double(result))
        } else {
            None
        }
    } else {
        if let Ok(result) = i16::from_str_radix(&tmp_string.to_string(), 10) {
            Some(GwToken::Integer(result))
        } else {            
            None
        }
    }
}


fn recognize_float_number_str<'a>(iterator : &mut PushbackCharsIterator<'a>) -> Option<GwToken> {
    if let Some(c) = iterator.next()  {
        let mut has_dot = false;
        if c.is_digit(10) || c == '.'  {
            if c == '.' {
                has_dot = true;
            }
            let mut tmp_string = String::new();
            tmp_string.push(c);
            loop {
                if let Some(c) = iterator.next() {
                    if c.is_digit(10) {
                        tmp_string.push(c);
                    } else if c == '.' && !has_dot {
                        tmp_string.push(c);
                        has_dot = true;
                    } else {
                        iterator.push_back(c);
                        break;
                    }
                } else {
                    break;
                }
            }
            convert_numeric_string(&tmp_string.to_string())
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
        } else if let Some(number_tok) = recognize_float_number_str(&mut self.chars_iterator) {
            return Some(number_tok);
        } else if let Some(string_literal) = recognize_string_literal(&mut self.chars_iterator) {
            return Some(GwToken::String(string_literal));
        } else if recognize_specific_char(&mut self.chars_iterator, '+') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::PlusTok));
        } else if recognize_specific_char(&mut self.chars_iterator, '^') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::PowOperatorTok));
        } else if recognize_specific_char(&mut self.chars_iterator, '/') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::DivTok));            
        } else if recognize_specific_char(&mut self.chars_iterator, '-') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::MinusTok));
            
        } else if recognize_specific_char(&mut self.chars_iterator, '=') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::EqlTok));                        
        } else if recognize_specific_char(&mut self.chars_iterator, '*') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::TimesTok));
        } else if recognize_specific_char(&mut self.chars_iterator, '(') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::LparTok));
        } else if recognize_specific_char(&mut self.chars_iterator, ')') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::RparTok));                        
        } else if recognize_specific_char(&mut self.chars_iterator, ':') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::ColonSeparatorTok));
        } else if recognize_specific_char(&mut self.chars_iterator, '<')
                  && recognize_specific_char(&mut self.chars_iterator, '>') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::DifferentTok));    
        } else if recognize_specific_char(&mut self.chars_iterator, ';') {
            return Some(GwToken::Keyword(tokens::GwBasicToken::SemiColonSeparatorTok));
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

    fn consume_rest_of_line(&mut self) -> String {
        let mut result = String::new();
        while let Some(next_char) = self.chars_iterator.next() {
            result.push(next_char);
        }
        return result;
    }

}
		  
pub enum ParserResult<T> {
    Success(T),
    Error(String),
    Nothing
}

pub enum IdExpressionResult<T> {
    Var(GwVariableExpression),
    Arr(GwCall),
    Builtin(Box<dyn GwExpression>),
    Error(ParserResult<T>)
}

fn is_builtin_function(func_name: &String) -> bool {
    match func_name.as_str() {
        "LOG" => true,
        "INT" => true,
        _ => false        
    }
}
fn create_builtin_function(func_name: &String, args: Vec<Box<dyn GwExpression>>)
                           -> Option<Box<dyn GwExpression>> {
    let mut mut_args = args;
    match func_name.as_str() {
        "LOG" if mut_args.len() == 1 => Some(Box::new(GwLog { expr: mut_args.remove(0) })),
        "INT" if mut_args.len() == 1 => Some(Box::new(GwInt { expr: mut_args.remove(0) })),        
        _ => None
    }   
}

fn parse_id_expression_aux<'a, T>(iterator : &mut PushbackTokensIterator<'a>,
                           id : String)
			      -> IdExpressionResult<T>

{
    if let Some(next_token) = iterator.next() {
        if let GwToken::Keyword(tokens::GwBasicToken::LparTok)  = next_token {
            let array_indices_result = 
                parse_with_separator(iterator,
                                     parse_expression,
                                     tokens::GwBasicToken::CommaSeparatorTok);    
            if let ParserResult::Success(array) = array_indices_result {
                if let Some(GwToken::Keyword(tokens::GwBasicToken::RparTok))  = iterator.next() {
                    if is_builtin_function(&id) {
                        return IdExpressionResult::Builtin(create_builtin_function(&id, array).expect("Builtin function not created"))
                    } else {
                        return IdExpressionResult::Arr(
                            GwCall {
                                array_or_function: id,
                                arguments: array
                            } );
                    }
                } else {
                    return IdExpressionResult::Error(ParserResult::Error(String::from("Right parenthesis expected")));
                }
            } else {
                return IdExpressionResult::Error(ParserResult::Error(String::from("Error parsing  array indices")));
            }            
        } else {
            iterator.push_back(next_token);
        }
    }
    return IdExpressionResult::Var(GwVariableExpression::with_name(id))
}

fn parse_id_expression<'a>(iterator : &mut PushbackTokensIterator<'a>,
                           id : String)
			   -> ParserResult<Box<dyn GwExpression>> {
    match parse_id_expression_aux(iterator, id) {
	IdExpressionResult::Var(variable) => ParserResult::Success(Box::new(variable)),
	IdExpressionResult::Arr(arr_access) => ParserResult::Success(Box::new(arr_access)),
        IdExpressionResult::Builtin(builtin) => ParserResult::Success(builtin),
	IdExpressionResult::Error(err) => err
    }
}


fn parse_restrict_identifier_expression(iterator : &mut PushbackTokensIterator)
                                        -> ParserResult<Box<dyn GwAssignableExpression>> {
    //    let tmp: Box<dyn GwAssignableExpression> = Box::new(GwVariableExpression::with_name(String::from("a")));
    //    let mut tmp = Box::new(GwVariableExpression::with_name(String::from("a")));
    let mut result = ParserResult::Nothing;
    if let Some(next_token) = iterator.next() {
        if let GwToken::Identifier(id) = next_token {

            match parse_id_expression_aux::<Box<dyn GwAssignableExpression>>(iterator, id) {
                IdExpressionResult::Var(variable) => {
		    let tmp : Box<dyn GwAssignableExpression> =
			Box::new(variable);
		    result = ParserResult::Success(tmp);
		}
                IdExpressionResult::Arr(arr_access) =>{
		    let tmp : Box<dyn GwAssignableExpression> =
			Box::new(arr_access);
		    result = ParserResult::Success(tmp);
		},
                IdExpressionResult::Builtin(_) => { result = ParserResult::Error(String::from("Cannot assign to builtin function")) },
                IdExpressionResult::Error(err) => { result = err;}
            }
        }
    }
    result
}

pub fn parse_single_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                   -> ParserResult<Box<dyn GwExpression>> {
    if let Some(next_token) = iterator.next() {
        if let GwToken::Identifier(id) = next_token {
            return parse_id_expression(iterator, id);
        } else if let GwToken::Integer(i_val) = next_token {
            return ParserResult::Success(Box::new(GwIntegerLiteral::with_value(i_val)))
        } else if let GwToken::Double(d_val) = next_token {
            return ParserResult::Success(Box::new(GwDoubleLiteral::with_value(d_val)))                
        } else if let GwToken::String(str_val) = next_token {
            return ParserResult::Success(Box::new(GwStringLiteral::with_value(str_val)))
        } else if let GwToken::Keyword(tokens::GwBasicToken::LparTok) = next_token {
            return parse_parenthesized_expression(iterator);
        } else if let GwToken::Keyword(tokens::GwBasicToken::MinusTok) = next_token {
            return parse_negation_expression(iterator);
        } else {
            iterator.push_back(next_token);
        }
    }
    ParserResult::Nothing
}

fn parse_parenthesized_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                      -> ParserResult<Box<dyn GwExpression>> {
    if let ParserResult::Success(inner_expr) = parse_expression(iterator) {
        if let Some(GwToken::Keyword(tokens::GwBasicToken::RparTok)) = iterator.next() {
            return ParserResult::Success(Box::new(GwParenthesizedExpr::new(inner_expr)));
        } else {
            return ParserResult::Error(String::from("Expecting right side parenthesis"));
        }
    } else {
        return ParserResult::Error(String::from("Expecting expression on parenthesized expression"));
    }
}

fn parse_negation_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                      -> ParserResult<Box<dyn GwExpression>> {
    if let ParserResult::Success(inner_expr) = parse_single_expression(iterator) {
        return ParserResult::Success(
            Box::new(GwNegExpr {
                expr: inner_expr
            } ));
    } else {
        return ParserResult::Error(String::from("Expecting single expression on negation"));
    }
}


fn one_kw_token_of<'a>(token : &'a GwToken, t1 : &'a tokens::GwBasicToken, t2 : &'a tokens::GwBasicToken) -> Option<&'a tokens::GwBasicToken>{
    match token  {
        GwToken::Keyword(tok) if *tok == *t1 || *tok == *t2 => Some(tok),
        _ => None
    }        
}

fn one_kw_token_of3<'a>(token : &'a GwToken,
                       t1 : &'a tokens::GwBasicToken,
                       t2 : &'a tokens::GwBasicToken,
                       t3 : &'a tokens::GwBasicToken)
                       -> Option<&'a tokens::GwBasicToken>{
    match token  {
        GwToken::Keyword(tok) if *tok == *t1 || *tok == *t2 || *tok == *t3 => Some(tok),
        _ => None
    }        
}


////
pub fn parse_multiplicative_expressions<'a>(iterator : &mut PushbackTokensIterator<'a>, current : Box<dyn GwExpression>)
                                          -> ParserResult<Box<dyn GwExpression>> {

  let mut current_expr = current;
  loop {
     if let Some(next_token) = iterator.next() {
         if let Some(tok) = one_kw_token_of3(&next_token, &tokens::GwBasicToken::DivTok, &tokens::GwBasicToken::TimesTok, &tokens::GwBasicToken::PowOperatorTok) {
             if let ParserResult::Success(right_side_parse_result) = parse_multiplicative_expression(iterator) {
                let kind = get_operation_kind_from_token(tok).unwrap();
                current_expr = 
                         Box::new(
                            GwBinaryOperation::new(kind, current_expr, right_side_parse_result ));
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
fn get_operation_kind_from_token(token : &tokens::GwBasicToken)
                                 -> Option<GwBinaryOperationKind> {
    match token {
        tokens::GwBasicToken::PlusTok => Some(GwBinaryOperationKind::Plus),
        tokens::GwBasicToken::MinusTok => Some(GwBinaryOperationKind::Minus),
        tokens::GwBasicToken::TimesTok => Some(GwBinaryOperationKind::Times),
        tokens::GwBasicToken::DivTok => Some(GwBinaryOperationKind::FloatDiv),
        tokens::GwBasicToken::EqlTok => Some(GwBinaryOperationKind::Equal),
        tokens::GwBasicToken::DifferentTok => Some(GwBinaryOperationKind::Different),	
        tokens::GwBasicToken::PowOperatorTok => Some(GwBinaryOperationKind::Exponent),        
        _ => None
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
                            GwBinaryOperation::new(
                                get_operation_kind_from_token(tok).unwrap(),
                                current_expr,
                                right_side_parse_result ));
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


fn is_comparison_operation_token(token : &GwToken)
                                 -> Option<&tokens::GwBasicToken> {
    match token {
        GwToken::Keyword(tok@tokens::GwBasicToken::EqlTok) =>
            Some(tok),
        GwToken::Keyword(tok@tokens::GwBasicToken::DifferentTok) =>
            Some(tok),
        _ => None	    
    }
}

pub fn parse_comparison_expressions<'a>(iterator : &mut PushbackTokensIterator<'a>,
                                        current : Box<dyn GwExpression>)
                                        -> ParserResult<Box<dyn GwExpression>> {

  let mut current_expr = current;
  loop {
     if let Some(next_token) = iterator.next() {
         if let Some(tok) = is_comparison_operation_token(&next_token) {
             if let ParserResult::Success(right_side_parse_result) = parse_additive_expression(iterator) {
                current_expr = 
                         Box::new(
                            GwBinaryOperation::new(
                                get_operation_kind_from_token(&tok).unwrap(),
                                current_expr,
                                right_side_parse_result ));
             } else {
                 return ParserResult::Error(String::from("Error parsing comparison expression, expecting right side operand "));
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


pub fn parse_comparison_expression<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                     -> ParserResult<Box<dyn GwExpression>> {

    match parse_additive_expression(iterator) {
        ParserResult::Success(left_side_parse_result) => {
            return parse_comparison_expressions(iterator, left_side_parse_result);
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
    return parse_comparison_expression(iterator);
}


fn parse_list_stat<'a>(_iterator : &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(
        GwListStat {}
    ));
}


fn parse_run_stat<'a>(_iterator : &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(
        GwRunStat {}
    ));
}

fn parse_system_stat<'a>(_iterator : &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(
        GwSystemStat {}
    ));
}


fn parse_print_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    let mut is_using = false;
    if let Some(tok) = iterator.next() {
        if let GwToken::Keyword(tokens::GwBasicToken::UsingTok) = tok {
            is_using = true;
        } else {
            iterator.push_back(tok);
        }
    } 
    
    if let ParserResult::Success(exprs) =
        parse_with_flexible_separator(
            iterator,
            parse_expression) {
            if is_using  {
                return ParserResult::Success(Box::new(
                    GwPrintUsingStat {
                        expressions: exprs
                    }
                ));                
            } else {
                return ParserResult::Success(Box::new(
                    GwPrintStat {
                        expressions: exprs
                    }
                ));
            }            
    } else {
        return ParserResult::Error(String::from("Expecting expression as PRINT argument"));
    }
}

fn parse_load_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    if let ParserResult::Success(expr) = parse_expression(iterator) {
        return ParserResult::Success(Box::new(
            GwLoadStat {
                filename: expr
            }
        ));
    } else {
        return ParserResult::Error(String::from("Expecting expression as LOAD  argument"));
    }
}


macro_rules! parse_seq {
    // default rule (the last one)
    ($iterator: expr, {}, $action:block) => {
	//return ParseError:Error(String::from($default_error_message));
	$action
    };
    // macro_rule case for expecting a token
    ($iterator: expr,  {
	token($token_pattern:pat, $error_message:expr);
	$($tail:tt)*
    }, $action:block) => {
	match $iterator.next() {
	    Some($token_pattern) => 
		parse_seq!($iterator, { $($tail)* },$action),
	    
	    _ => {  return ParserResult::Error(String::from($error_message)); }
	}
    };


    ($iterator: expr,  {
	opt_token($token_pattern:pat,
		  $result_id:ident = $opt_result:expr);
	$($tail:tt)*
    }, $action:block) => {
	let $result_id = match $iterator.next() {
	    Some($token_pattern) => Some($opt_result),
	    Some(token) => {  $iterator.push_back(token); None }
	    _ => { None }
	};
	parse_seq!($iterator, { $($tail)* } , $action);
    };
    
    // macro_rule case for parsing inner element
    ($iterator: expr, {
	parse_success($pattern:pat, $parse_expr:expr);
	$($tail:tt)*	
    },
     $action: block) => {
	match $parse_expr {
	    ParserResult::Success($pattern) => 
		parse_seq!($iterator, {$($tail)*}, $action),	    
	    ParserResult::Error(error) => {
		return ParserResult::Error(error);
	    }
	    ParserResult::Nothing => {
		return ParserResult::Error(String::from("Nothing!"));
	    }
	}
    };
}


fn parse_while_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
			-> ParserResult<Box<dyn GwInstruction>> {

    parse_seq![
	iterator,
	{
	    parse_success(condition, parse_expression(iterator));
	},
	{
	    return ParserResult::Success(
		Box::new(
		    GwWhile {
			condition
		    }
		)
	    );
	}
    ]
}

fn parse_for_stat<'a>(iterator: &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    parse_seq![
        iterator,
        {
            token(GwToken::Identifier(variable), "Expecting identifier");
            token(GwToken::Keyword(tokens::GwBasicToken::EqlTok), "Expecting equal (=)");
            parse_success(from, parse_expression(iterator));
            token(GwToken::Keyword(tokens::GwBasicToken::ToTok), "Expecting 'to'");
            parse_success(to_expr, parse_expression(iterator));            
        },
        {
            ParserResult::Success(
                Box::new(
                    GwFor {
                        variable,
                        from,
                        to: to_expr,
                        step: None
                    }
                ))
        }

    ]
}

fn parse_next_stat<'a>(_iterator : &mut PushbackTokensIterator<'a>)
		       -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(GwNext{ variable: None}));    
}

fn parse_wend_stat<'a>(_iterator : &mut PushbackTokensIterator<'a>)
		       -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(GwWend{}));    
}

fn parse_input_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                        -> ParserResult<Box<dyn GwInstruction>> {
    parse_seq![
	iterator,
	{
	opt_token(GwToken::String(prompt_txt), prompt = prompt_txt);
	opt_token(GwToken::Keyword(tokens::GwBasicToken::CommaSeparatorTok), _x = 1);
	parse_success(input_vars, parse_with_separator(iterator,
				     parse_restrict_identifier_expression,
						       tokens::GwBasicToken::CommaSeparatorTok));
	},
	{
	    return ParserResult::Success(Box::new(
		    GwInputStat {
			prompt: prompt,
			variables: input_vars
		    }
		));
	}
    ];
    /*
    let next_token = iterator.next();
    if let Some(GwToken::String(prompt_txt)) = next_token {
	if let Some(GwToken::Keyword(tokens::GwBasicToken::CommaSeparatorTok)) = iterator.next() {
	    let input_vars_result =
		parse_with_separator(iterator,
				     parse_restrict_identifier_expression,
				     tokens::GwBasicToken::CommaSeparatorTok);
	    if let ParserResult::Success(input_vars) = input_vars_result {
		return ParserResult::Success(Box::new(
		    GwInputStat {
			prompt: Some(prompt_txt),
			variables: input_vars
		    }
		));
	    } else {
		return ParserResult::Error(String::from("??"));
	    }
	} else {
	    return ParserResult::Error(String::from("Expecting comma"));
	}
    } else if let Some(GwToken::Identifier(id)) = next_token {
        return ParserResult::Success(Box::new(
            GwInputStat {
		prompt: None,
                variables: vec![]//inputVars
            }
        ));
    } else {
        return ParserResult::Error(String::from("Expecting variable as INPUT argument"));
    }*/
}

fn parse_if_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                     -> ParserResult<Box<dyn GwInstruction>> {

    if let ParserResult::Success(expr) =  parse_expression(iterator) {
        if let Some(GwToken::Keyword(tokens::GwBasicToken::ThenTok)) = iterator.next() {
            if let Some(GwToken::Integer(line_number)) = iterator.next() {
                return ParserResult::Success(Box::new(
                    GwIf::new(expr, line_number)
                ));                
            } else {
                return ParserResult::Error(String::from("Expecting line number"));
            }
        } else {
            return ParserResult::Error(String::from("Expecting THEN keyword"));                    
        }
    } else {
        return ParserResult::Error(String::from("Expecting variable as expression as part of IF statement"));        
    }
}

fn parse_var_range_fragment<'a>(iterator : &mut PushbackTokensIterator<'a>,
                      start : char)
                      -> ParserResult<DefVarRange> {
    let token_result = iterator.next();
    if let Some(tok) = token_result {
        match tok {
            GwToken::Keyword(tokens::GwBasicToken::MinusTok) => {
                if let Some(GwToken::Identifier(end)) = iterator.next() {
                    //TODO unsafe way of getting single char
                    // also no validation on that the variable has
                    // to be a single char
                    return ParserResult::Success(
                        DefVarRange::Range(start, end.chars().nth(0).unwrap())
                    );
                } else {
                    return ParserResult::Error(String::from("Expecting end of range "));
                }
            }
            _ => {
                iterator.push_back(tok);
                return ParserResult::Success(DefVarRange::Single(start))
            }
        }
    } else {
        return  ParserResult::Success(DefVarRange::Single(start))
    }
}


fn parse_var_range<'a>(iterator : &mut PushbackTokensIterator<'a>)
                   -> ParserResult<DefVarRange> {
    let token_result = iterator.next();
    if let Some(tok) = token_result {
        match tok {
            GwToken::Identifier(start) => {
                //TODO UNSAFE CHAR access
                // also we need to validate for single chars
                return parse_var_range_fragment(
                    iterator,
                    start.chars().nth(0).unwrap());
            }
            _ => {
                iterator.push_back(tok);
                return ParserResult::Nothing;
            }
        }
    } else {
        return ParserResult::Nothing;
    }
}


fn parse_defdbl_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                         -> ParserResult<Box<dyn GwInstruction>> {
    let ranges_result =
        parse_with_separator(iterator,
                             parse_var_range,
                             tokens::GwBasicToken::CommaSeparatorTok);
    match ranges_result {
        ParserResult::Success(ranges_vector) => {
            return ParserResult::Success(Box::new(
                GwDefDbl::with_var_range(ranges_vector)
            ));
        }, 
        ParserResult::Error(error) => {
            return ParserResult::Error(error)
        }
        ParserResult::Nothing => ParserResult::Nothing
    }
}

fn parse_rem_stat<'a>(iterator : &mut PushbackTokensIterator<'a>)
                      -> ParserResult<Box<dyn GwInstruction>> {
    return ParserResult::Success(Box::new(
        GwRem {
            comment: iterator.consume_rest_of_line()
        }
        ));
}


fn parse_cls_stat<'a>(_iterator : &mut PushbackTokensIterator<'a>)
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


fn parse_with_separator<'a, F,T>(
    iterator : &mut PushbackTokensIterator<'a>,
    item_parse_fn  : F,
    separator : tokens::GwBasicToken) ->
     ParserResult<Vec<T>>
  where F : Fn(&mut PushbackTokensIterator<'a>) -> ParserResult<T>  {
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

//
fn parse_with_flexible_separator<'a, F,T>(
    iterator : &mut PushbackTokensIterator<'a>,
    item_parse_fn  : F) ->
     ParserResult<Vec<(Option<T>, Option<PrintSeparator>)>>
  where F : Fn(&mut PushbackTokensIterator<'a>) -> ParserResult<T>  {
    let mut result = Vec::new();
    loop {
        let mut item_result: Option<T> = None;
        match item_parse_fn(iterator) {
            ParserResult::Success(item) => {
                item_result = Some(item);
            },
            ParserResult::Error(error) => {
                return ParserResult::Error(error);
            },
            ParserResult::Nothing => {
            }
        }
        let next_token = iterator.next();
        match (next_token, &item_result) {
            (Some(GwToken::Keyword(tokens::GwBasicToken::SemiColonSeparatorTok)), _) => {
                result.push((item_result, Some(PrintSeparator::Semicolon)));
                continue;
            }
            (Some(GwToken::Keyword(tokens::GwBasicToken::CommaSeparatorTok)), _) => {
                result.push((item_result, Some(PrintSeparator::Comma)));
                continue;
            }
            (Some(next@GwToken::Keyword(tokens::GwBasicToken::ColonSeparatorTok)), None) => { 
                iterator.push_back(next);
                break;
            },
            (Some(next@GwToken::Keyword(tokens::GwBasicToken::ColonSeparatorTok)), _) => { 
                iterator.push_back(next);
                result.push((item_result, None));                
                break;
            },
            (Some(token_result), _) => {
                iterator.push_back(token_result);
                result.push((item_result, None));
                continue;
            },
            (_, Some(_)) => {
                result.push((item_result, None));
                break;
            }
            (_, None) => break,            
        }
    }
    return ParserResult::Success(result);
}

//

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

fn parse_array_assignemnt<'a>(
    iterator : &mut PushbackTokensIterator<'a>,
    identifier : String)
    -> ParserResult<Box<dyn GwInstruction>> {
    let array_indices_result = 
        parse_with_separator(iterator,
                             parse_expression,
                             tokens::GwBasicToken::CommaSeparatorTok);    
    if let ParserResult::Success(array) = array_indices_result {
        if let Some(GwToken::Keyword(tokens::GwBasicToken::RparTok))  = iterator.next() {
            if let Some(GwToken::Keyword(tokens::GwBasicToken::EqlTok))  = iterator.next() {
                if let ParserResult::Success(expr) =  parse_expression(iterator) {
                    return ParserResult::Success(
                        Box::new(
                            GwArrayAssign {
                                variable: identifier,
                                indices_expressions: array,
                                expression: expr
                            }));
                } else {
                    return ParserResult::Error(String::from("Error expression expected"));
                }                
            } else {
                return ParserResult::Error(String::from("Equal  expected"));
            }            
        } else {
            return ParserResult::Error(String::from("Right parenthesis expected"));
        }
    } else {
        return ParserResult::Error(String::from("Error parsing  array indices"));
    }
}
    

fn parse_assignment<'a>(iterator : &mut PushbackTokensIterator<'a>, identifier : String)
                        -> ParserResult<Box<dyn GwInstruction>> {

    if let Some(next_token)  = iterator.next() {
        if let GwToken::Keyword(tokens::GwBasicToken::LparTok) = next_token {
             return parse_array_assignemnt(iterator, identifier);            
        } else if let GwToken::Keyword(tokens::GwBasicToken::EqlTok) = next_token {
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
            GwToken::Keyword(tokens::GwBasicToken::RemTok) => parse_rem_stat(iterator),            
            GwToken::Keyword(tokens::GwBasicToken::DefdblTok) => parse_defdbl_stat(iterator),            
            GwToken::Keyword(tokens::GwBasicToken::ColorTok) => parse_color_stat(iterator),                        
            GwToken::Keyword(tokens::GwBasicToken::KeyTok) => parse_key_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::PrintTok)  => parse_print_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::ListTok)  => parse_list_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::RunTok)  => parse_run_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::LoadTok)  => parse_load_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::IfTok)  => parse_if_stat(iterator),                        
            GwToken::Keyword(tokens::GwBasicToken::SystemTok)  => parse_system_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::InpTok)  => parse_input_stat(iterator),
	    GwToken::Keyword(tokens::GwBasicToken::WhileTok) => parse_while_stat(iterator),
	    GwToken::Keyword(tokens::GwBasicToken::WendTok) => parse_wend_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::ForTok) => parse_for_stat(iterator),
            GwToken::Keyword(tokens::GwBasicToken::NextTok) => parse_next_stat(iterator),
	    
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

pub fn parse_repl_instruction_string(line : String)
                                          -> ParserResult<Box<dyn GwInstruction>> {
    
    let pb = PushbackCharsIterator {
        chars: line.chars(),
        pushed_back: None
    };
    let mut tokens_iterator = PushbackTokensIterator::create(pb);
    parse_instruction(&mut tokens_iterator)
}


pub fn parse_instruction_line_from_string(line : String)
                                          -> ParserResult<ProgramLine> {
    
    let pb = PushbackCharsIterator {
        chars: line.chars(),
        pushed_back: None
    };
    let mut tokens_iterator = PushbackTokensIterator::create(pb);
    parse_instruction_line(&mut tokens_iterator)
}

fn parse_same_line_instruction_sequence<'a>(iterator : &mut PushbackTokensIterator<'a>)
                                            -> ParserResult<Vec<Box<dyn GwInstruction>>> {
    if let Some(next_tok) = iterator.next() {
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
            let parse_result = parse_instruction(iterator);
            if let ParserResult::Success(instr) = parse_result {
                match parse_same_line_instruction_sequence(iterator) {
                 ParserResult::Success(rest_inst) => {
                    return ParserResult::Success(
                        ProgramLine {
                            line : line_number,
                            instruction : instr,
                            rest_instructions : Some(rest_inst)
                        }
                    );
                 },
                 ParserResult::Nothing => {                    
                    return ParserResult::Success(
                        ProgramLine {
                            line : line_number,
                            instruction : instr,
                            rest_instructions : None
                        }
                    );
                 },
                 ParserResult::Error(err) => 
                       ParserResult::Error(String::from(err))
                }
            } else {
                if let ParserResult::Error(msg) = parse_result {
                    ParserResult::Error(msg)
                } else {
                    ParserResult::Error(String::from("Error parsing line"))
                }
            }
        } else {
            ParserResult::Error(String::from("line number expected"))
        }
    } else {
        ParserResult::Nothing
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
        let pb = PushbackCharsIterator {
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
    fn it_parses_different() {
        let str = "ab <> bc";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(AB <> BC)"));
            }
            _ => panic!("errror")
        }        
    }


    #[test]
    fn it_parses_minus() {
        let str = "ab - bc";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(AB - BC)"));
            }
            _ => panic!("errror")
        }        
    }

    
    #[test]
    fn it_parses_assignment_instruction() {
        let str = "10 x = ab";
        let pb = PushbackCharsIterator {
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
    fn it_parses_input_no_prompt() -> Result<(), String> {
        let str = "10 INPUT a";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_instruction_line(&mut tokens_iterator) {
            ParserResult::Success(instr) => {
                let mut buf = String::new();
                instr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(10 INPUT A)"));
		return Ok(());
            }
            ParserResult::Error(error) => { return Err(error); }
	    ParserResult::Nothing => { return Err("Nothing".to_string()); }
        }        
    }
    
    #[test]
    fn it_parses_input() -> Result<(), String> {
        let str = "10 INPUT \"hello?\",a,b,c";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_instruction_line(&mut tokens_iterator) {
            ParserResult::Success(instr) => {
                let mut buf = String::new();
                instr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("(10 INPUT \"hello?\",A,B,C)"));
		return Ok(());
            }
            ParserResult::Error(error) => { return Err(error); }
	    ParserResult::Nothing => { return Err("Nothing".to_string()); }
        }        
    }


    #[test]
    fn it_parses_mult_instruction_line() {
        let str = "10 x = ab : y = bc : z = cd";
        let pb = PushbackCharsIterator {
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
        let pb = PushbackCharsIterator {
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
        let pb = PushbackCharsIterator {
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

    //
    #[test]
    fn it_parses_a_sequence_with_flexible_separator() {
        let str = "ab ; 12 ;; a + cd;";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_with_flexible_separator(&mut tokens_iterator,
                                   parse_expression)  {
            ParserResult::Success(vect) => {
                match vect[..] {
                    [(Some(ref arg1), Some(PrintSeparator::Semicolon)),
                     (Some(ref arg2), Some(PrintSeparator::Semicolon)),
                     (None,           Some(PrintSeparator::Semicolon)),
                     (Some(ref arg3), Some(PrintSeparator::Semicolon))
                    ] => {
                        let mut buf = String::new();
                        arg1.fill_structure_string(&mut buf);
                        assert_eq!(buf, "AB");
                        buf.clear();
                        arg2.fill_structure_string(&mut buf);
                        assert_eq!(buf, "12");
                        buf.clear();
                        arg3.fill_structure_string(&mut buf);
                        assert_eq!(buf, "(A + CD)");
                    }
                    _ => { assert!(false); }
                }
            },
            _ => assert!(false)
        }                
    }

    #[test]
    fn it_parses_a_sequence_for_print_separators() {
        let str = "ab , 1 2 cd";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_with_flexible_separator(&mut tokens_iterator,
                                            parse_expression)  {
            ParserResult::Success(vect) => {
                match vect[..] {
                    [(Some(ref arg1), Some(PrintSeparator::Comma)),
                     (Some(ref arg2), None),
                     (Some(ref arg3), None),
                     (Some(ref arg4), None)
                    ] => {
                        let mut buf = String::new();
                        arg1.fill_structure_string(&mut buf);
                        assert_eq!(buf, "AB");
                        buf.clear();
                        arg2.fill_structure_string(&mut buf);
                        assert_eq!(buf, "1");
                        buf.clear();
                        arg3.fill_structure_string(&mut buf);
                        assert_eq!(buf, "2");
                        buf.clear();
                        arg4.fill_structure_string(&mut buf);
                        assert_eq!(buf, "CD");                        
                    }
                    _ => { assert!(false); }
                }
            },
            _ => assert!(false)
        }                
    }

    #[test]
    fn it_parses_one_element_print() {
        let str = "ab";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_with_flexible_separator(&mut tokens_iterator,
                                            parse_expression)  {
            ParserResult::Success(vect) => {
                match vect[..] {
                    [(Some(ref arg1), None)] => {
                        let mut buf = String::new();
                        arg1.fill_structure_string(&mut buf);
                        assert_eq!(buf, "AB");
                    }
                    _ => {  assert!(false); }
                }
            },
            _ => assert!(false)
        }                
    }
    
    #[test]
    fn it_parses_empty_sequence_for_print() {
        let str = " ";
        let pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_with_flexible_separator(&mut tokens_iterator,
                                            parse_expression)  {
            ParserResult::Success(vect) => {
                match vect[..] {
                    [] => { assert!(true); }
                    _ => {  assert!(false); }
                }
            },
            _ => assert!(false)
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
    fn it_parser_negative_operator() {
        let str =  "-(1+1)";
        let  pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("-((1 + 1))"));
            }
            _ => panic!("errror")
        }                
    }

    #[test]
    fn it_parser_call_expression() {
        let str =  "FNFUNC(1,2)";
        let  pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_expression(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("FNFUNC(1,2)"));
            }
            _ => panic!("errror")
        }                
    }

    

    #[test]
    fn it_parses_array_assignment() {
        let str = "myarr(x + 1, otherarr(30)) = 20";
        let  pb = PushbackCharsIterator {
            chars: str.chars(),
            pushed_back: None
        };
        let mut tokens_iterator = PushbackTokensIterator::create(pb);
        match parse_instruction(&mut tokens_iterator) {
            ParserResult::Success(expr) => {
                let mut buf = String::new();
                expr.fill_structure_string(&mut buf);
                assert_eq!(buf, String::from("MYARR((X + 1)OTHERARR(30)) = 20"));
            }
            _ => panic!("errror")
        }        
    }

    #[test]
    fn it_parses_if_stat() {
	let str = "10 IF x = 1 THEN 30";
	let pb = PushbackCharsIterator::new(str.chars());
	let mut tokens_iterator = PushbackTokensIterator::create(pb);
	match parse_instruction_line(&mut tokens_iterator) {
	    ParserResult::Success(instr) => {
		let mut buf = String::new();
		instr.fill_structure_string(&mut buf);
		println!("{}", buf);
		assert_eq!(buf, "(10 IF (X = 1) 30)");
	    }
	    _ => panic!("IF not parsed!")
	}
    }

    #[test]
    fn it_parses_while_stat() {
	let str = "10 WHILE x = 1";
	let pb = PushbackCharsIterator::new(str.chars());
	let mut tokens_iterator = PushbackTokensIterator::create(pb);
	match parse_instruction_line(&mut tokens_iterator) {
	    ParserResult::Success(instr) => {
		let mut buf = String::new();
		instr.fill_structure_string(&mut buf);
		println!("{}", buf);
		assert_eq!(buf, "(10 WHILE (X = 1))");
	    }
	    _ => panic!("WHILE not parsed!")
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
