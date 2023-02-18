use super::{
    EvaluationContext,
    GwAssignableExpression,
    GwInstruction,
    GwProgram,
    InstructionResult,
    LineExecutionArgument
};

// SWAP statement implementation
//
// This statement swaps the value of
// two variables or array access expr.
// For example:
//
//  ```
//   SWAP X,Y
//  ```
pub struct GwSwap {
    left: Box<dyn GwAssignableExpression>,
    right: Box<dyn GwAssignableExpression>,
}

impl GwSwap {
    pub fn new(left: Box<dyn GwAssignableExpression>,
               right: Box<dyn GwAssignableExpression>) -> GwSwap {
        GwSwap {
            left, right
        }
    }
}

impl GwInstruction for GwSwap {
    fn eval (&self,
             _line: i16,
             _argument: LineExecutionArgument,
             context : &mut EvaluationContext,
             program: &mut GwProgram) -> InstructionResult {

        match (self.left.eval(context), self.right.eval(context)) {
            (Ok(result1), Ok(result2)) => {
                check_result![ self.left.assign_value(result2, context) ];
                check_result![ self.right.assign_value(result1, context) ];
            }
            (Err(error), _) => {
                return InstructionResult::EvaluateToError(error.to_string());
            }
            (_, Err(error)) => {
                return InstructionResult::EvaluateToError(error.to_string());
            }
        }
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("SWAP ");
        self.left.fill_structure_string(buffer);
        buffer.push_str(", ");
        self.right.fill_structure_string(buffer);
    }
}



#[cfg(test)]
mod swap_tests {

    use super::*;
    use crate::eval::{ExpressionEvalResult, GwVariableExpression };
    use crate::eval::eval_tests::{empty_context, empty_program};

    #[test]
    fn it_swaps_variables() {
        let mut program = empty_program();
        let mut ctx = empty_context();
        let stat =
            GwSwap::new(
                Box::new(GwVariableExpression::with_name("x".to_string())),
                Box::new(GwVariableExpression::with_name("y".to_string())));
        ctx.set_variable("x", &ExpressionEvalResult::IntegerResult(100)).expect("success");
        ctx.set_variable("y", &ExpressionEvalResult::IntegerResult(200)).expect("success");

        assert!(
            if let Some(ExpressionEvalResult::IntegerResult(100)) = ctx.lookup_variable("x")
            { true }
            else { false });
        assert!(
            if let Some(ExpressionEvalResult::IntegerResult(200)) = ctx.lookup_variable("y")
            { true }
            else { false });

        let eval_result = stat.eval(1,
                                    LineExecutionArgument::Empty,
                                    &mut ctx,
                                    &mut program);

        assert!(if let InstructionResult::EvaluateNext = eval_result { true } else { false });

        assert!(
            if let Some(ExpressionEvalResult::IntegerResult(200)) = ctx.lookup_variable("x")
            { true }
            else { false });
        assert!(
            if let Some(ExpressionEvalResult::IntegerResult(100)) = ctx.lookup_variable("y")
            { true }
            else { false });
    }

}
