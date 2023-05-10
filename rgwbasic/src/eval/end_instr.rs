use super::{ GwInstruction, InstructionResult, LineExecutionArgument, EvaluationContext, GwProgram };


pub struct GwEnd {
}

impl GwInstruction for GwEnd {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        _context: &mut EvaluationContext,
        _program: &mut GwProgram,
    ) -> InstructionResult {
        InstructionResult::EvaluateEnd
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"END");
    }
}
