use super::{ EvaluationContext, LineExecutionArgument,
             InstructionResult, GwInstruction };


/// AST element  for GOSUB subroutine invocation element
/// For example:
/// ```
/// GOSUB 100
/// ```
pub struct GwGosub {
    line_number: i16
}

impl GwGosub {
    pub fn new(line_number: i16) -> GwGosub {
        GwGosub { line_number }
    }
}

impl GwInstruction for GwGosub {
    fn eval (&self,
             line: i16,
             argument: LineExecutionArgument,
             context : &mut EvaluationContext) -> InstructionResult {
        if let LineExecutionArgument::SubReturn = argument {
            InstructionResult::EvaluateNext
        } else {
            if let Some(real_line) =  context.get_real_line(self.line_number) {
                context.push_return(line);
                InstructionResult::EvaluateLine(real_line)
            } else {
                InstructionResult::EvaluateToError("Unknown line".into())
            }
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("GOSUB ");
    }
}


/// AST element for the RETURN statement
/// For example
/// ```
/// RETURN
/// ```
pub struct GwReturn {
}

impl GwReturn {
    pub fn new() -> GwReturn {
        GwReturn {}
    }
}

impl GwInstruction for GwReturn {
    fn eval (&self,
             _line: i16,
             _argument: LineExecutionArgument,
             context : &mut EvaluationContext) -> InstructionResult {
        if let Some(line_to_return) = context.pop_return() {
            InstructionResult::EvaluateLineWithArg(
                line_to_return,
                LineExecutionArgument::SubReturn)
        } else {
            InstructionResult::EvaluateToError("RETURN: no place to return".into())
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("RETURN");
    }
}
