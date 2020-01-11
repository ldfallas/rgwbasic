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
}

pub struct GwStringLiteral {
    value : String
}

impl GwExpression for GwStringLiteral {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::StringResult(String::from(&self.value))
    }
}

pub struct GwIntegerLiteral {
    value : i16
}

impl GwExpression for GwIntegerLiteral {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::IntegerResult(self.value)
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


struct PushbackCharsIterator<'a> {
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

fn recognize_int_number_str<'a>(iterator : &mut PushbackCharsIterator<'a>) -> Option<u16> {
    if let Some(c) = iterator.next()  {
        if c.is_digit(10) {
            let mut tmp_string = String::new();
            tmp_string.push(c);
            loop {
                if let Some(c) = iterator.next() {
                    if (c.is_digit(10)) {
                        tmp_string.push(c);                        
                    } else {
                        iterator.push_back(c);
                        break;
                    }
                } else {
                    break;
                }
            }
            if let Ok(result) = u16::from_str_radix(&tmp_string.to_string(), 10) {
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


#[cfg(test)]
mod eval_tests {
    use std::collections::HashMap;
    
    use crate::gwparser::GwAssign;
    use crate::gwparser::GwIntegerLiteral;
    use crate::gwparser::ProgramLine;
    use crate::gwparser::GwProgram;
    use crate::gwparser::EvaluationContext;
    use crate::gwparser::ExpressionEvalResult;

    
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
    use crate::gwparser::PushbackCharsIterator;
    use crate::gwparser::consume_whitespace;
    use crate::gwparser::recognize_int_number_str;


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
}
