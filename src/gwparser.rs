use std::collections::HashMap;



pub enum InstructionResult {
    EvaluateNext,
    EvaluateLine(u16)
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
    lines_index : Vec<u16>
}


#[cfg(test)]
mod eval_tests {
    #[test]
    fn it_tests_basic_eval() {
        
        assert_eq!(23,3);
    }
}


#[cfg(test)]
mod parser_tests {
    #[test]
    fn it_tests() {
        assert_eq!(23,3);
    }
}
