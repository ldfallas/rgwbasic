use std::rc::Rc;
use super::{ EvaluationContext,
             ExpressionEvalResult,
             GwExpression,
             LineExecutionArgument,
             InstructionResult,
             GwInstruction,
             GwProgram };

pub struct GwIf {
    condition: Box<dyn GwExpression>,
    then_line: i16,
}

impl GwIf {
    pub fn new(condition: Box<dyn GwExpression>, then_line: i16) -> GwIf {
        return GwIf {
            condition,
            then_line,
        };
    }
}

fn is_false_result(result: &ExpressionEvalResult) -> bool {
    match result {
        ExpressionEvalResult::IntegerResult(i_result) if *i_result == 0 => true,
        ExpressionEvalResult::SingleResult(s_result) if *s_result == 0.0 => true,
        ExpressionEvalResult::DoubleResult(d_result) if *d_result == 0.0 => true,
        _ => false
    }
}

impl GwInstruction for GwIf {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        _program: &mut GwProgram,
    ) -> InstructionResult {
        match self.condition.eval(context) {
            Ok(eval_result) if is_false_result(&eval_result) => {
                InstructionResult::EvaluateNext
            }
            Ok(_) => {
                if let Some(real_line) = context.get_real_line(self.then_line) {
                    InstructionResult::EvaluateLine(real_line)
                } else {
                    panic!("Jumping to a non-existing line!");
                }
            }
            Err(err) => InstructionResult::EvaluateToError(err.into())
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"IF ");
        self.condition.fill_structure_string(buffer);
        buffer.push_str(format!(" {}", self.then_line).as_str());
    }
}


// Definition of IF with nested statements
// For example:
// ```
// IF X > 10 THEN PRINT "a" : PRINT "b"
//
pub struct GwIfWithStats {
    condition: Box<dyn GwExpression>,
    stats: Vec<Rc<dyn GwInstruction>>
}

impl GwIfWithStats {
    pub fn new(condition: Box<dyn GwExpression>,
               stats: Vec<Rc<dyn GwInstruction>>) -> GwIfWithStats {
        GwIfWithStats {
            condition,
            stats
        }
    }
}

impl GwInstruction for GwIfWithStats {
    fn eval (&self,
             line: i16,
             _argument: LineExecutionArgument,
             context : &mut EvaluationContext,
             program: &mut GwProgram) -> InstructionResult {
        match self.condition.eval(context) {
            Ok(eval_result) if is_false_result(&eval_result) => {
                InstructionResult::EvaluateNext
            }
            Ok(_) => {
                for stat in &self.stats {
                    stat.eval(line, LineExecutionArgument::Empty, context, program);
                }
                InstructionResult::EvaluateNext
            }
            Err(err) => InstructionResult::EvaluateToError(err.into())
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("IF ");
        self.condition.fill_structure_string(buffer);
        buffer.push_str(" THEN ");
        let mut c = self.stats.len();
        for stat in &self.stats {
            stat.fill_structure_string(buffer);
            c -= 1;
            if c != 0 {
                buffer.push_str(" : ");
            }
        }
           
    }
}

#[cfg(test)]
mod if_stat_tests {
    use super::*;
    use crate::eval::{GwIntegerLiteral, GwAssign};
    use crate::eval::eval_tests::{ empty_context, empty_program };
    
    #[test]
    fn it_executes_then_stats() -> Result<(), & 'static str>{
        let mut ctx = empty_context();
        let mut program = empty_program();
        let if_stat = GwIfWithStats::new(
            Box::new(GwIntegerLiteral::with_value(1)),
            vec![
                Rc::new(
                    GwAssign {
                        variable: "x".into(),
                        expression: Box::new(GwIntegerLiteral::with_value(123))
                    })
            ]
        );
        
        let eval_result =
            if_stat.eval(0, LineExecutionArgument::Empty, &mut ctx, &mut program);


        assert!(
            match ctx.lookup_variable("x") {
                Some(ExpressionEvalResult::SingleResult(value))  => {
                    assert_eq!(123, *value as i32);
                    true
                }
                _ => false
            });

        match eval_result {
            InstructionResult::EvaluateNext => Ok(()),
            _ => Err("Unexpected eval result")
        }     
    }

    #[test]
    fn it_do_not_execute_then_stats() -> Result<(), & 'static str>{
        let mut ctx = empty_context();
        let mut program = empty_program();
        let if_stat = GwIfWithStats::new(
            Box::new(GwIntegerLiteral::with_value(0)),
            vec![
                Rc::new(
                    GwAssign {
                        variable: "x".into(),
                        expression: Box::new(GwIntegerLiteral::with_value(123))
                    })
            ]
        );
        
        let eval_result =
            if_stat.eval(0,
                         LineExecutionArgument::Empty,
                         &mut ctx,
                         &mut program);


        assert!(
            match ctx.lookup_variable("x") {
                None => true,
                _ => false
            });

        match eval_result {
            InstructionResult::EvaluateNext => Ok(()),
            _ => Err("Unexpected eval result")
        }     
    }
}
