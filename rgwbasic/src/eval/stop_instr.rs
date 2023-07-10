use super::{ GwInstruction,
             InstructionResult,
             EvaluationContext,
             LineExecutionArgument,
             GwProgram };

/// AST element for the `STOP` statement
pub struct GwStop {
}

impl GwInstruction for GwStop {
    fn eval(&self,
            _line: i16,
            _argument: LineExecutionArgument,
            _context: &mut EvaluationContext,
            _program: &mut GwProgram) -> InstructionResult {
        // Return value to instruct the interpreter to stop the execution.
        InstructionResult::EvaluateEnd
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"STOP");
    }
}
