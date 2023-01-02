use super::{GwExpression, GwInstruction,
            InstructionResult, EvaluationContext, ExpressionEvalResult,
            LineExecutionArgument};

pub struct GwWhile {
    pub condition : Box<dyn GwExpression>,
}

impl GwInstruction for GwWhile {
    fn eval (&self,
             line: i16,
             _arg: LineExecutionArgument,
             context : &mut EvaluationContext) -> InstructionResult {
        let mut wend_line : i16 = 0;

        // Find the cached corresponding line for this WHILE statement
        if let Some(corresponding_wend) =  context.pair_instruction_table.get(&line) {
            wend_line = *corresponding_wend;
        } else if let Some(ref real_lines) = context.real_lines {
            // Try to look for the WEND statement in the program lines
            let index_of_wend = find_wend(line, real_lines);
            if index_of_wend == -1 {
                return InstructionResult::EvaluateToError(String::from("WHILE WITHOUT WEND"));
            } else {
                context.pair_instruction_table.insert(line, index_of_wend);
                context.pair_instruction_table.insert(index_of_wend, line);
            }
            wend_line = index_of_wend;
        }

        // Evaluate the condition and move the following line
        let condition_evaluation = self.condition.eval(context);
        match condition_evaluation {
            Ok(ExpressionEvalResult::IntegerResult(result)) if result == 0 => {
                InstructionResult::EvaluateLine(wend_line + 1)
            }
            Ok(ExpressionEvalResult::IntegerResult(_)) => {
                InstructionResult::EvaluateNext
            }
            _ => {
                InstructionResult::EvaluateToError(String::from("Type mismatch"))
            }
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"WHILE ");
        self.condition.fill_structure_string(buffer);
    }
    fn is_while(&self) -> bool { true }
}

fn find_wend(line: i16, real_lines: &Vec<&Box<dyn GwInstruction>>) -> i16 {
    let mut curr_line = line + 1;
    let mut while_end_balance = 0;
    loop {
        if curr_line >= real_lines.len() as i16 {
            break;
        } else if let Some(ref instr) = real_lines.get(curr_line as usize) {

            if instr.is_while() {
                while_end_balance += 1;
            }
            if instr.is_wend() {
                if while_end_balance  == 0 {
                    return curr_line as i16;
                } else {
                    while_end_balance -= 1;
                }
            }
        }
        curr_line += 1;
    }
    return -1;
}

pub struct GwWend {
}

impl GwInstruction for GwWend {
    fn eval (&self,
             line: i16,
             _arg: LineExecutionArgument,
             context : &mut EvaluationContext) -> InstructionResult{
        if let Some(corresponding_while) =  context.pair_instruction_table.get(&line) {
            InstructionResult::EvaluateLine(*corresponding_while)
        } else {
            InstructionResult::EvaluateToError(String::from("WEND WITHOUT WHILE"))
        }
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"WEND");
    }
    fn is_wend(&self) -> bool { true }
}


#[cfg(test)]
mod while_eval_tests {
    use crate::eval::eval_tests::DummyConsole;
    use crate::eval::*;
    use crate::eval::while_instr::*;

    #[test]
    fn it_iteratates_while_loop() {
        let mut ctxt = EvaluationContext::new(Box::new(DummyConsole{}));
        let w = GwWhile {
            condition: Box::new(GwVariableExpression { name: String::from("x") })
        };

        let assign_result = ctxt.set_variable(
            &String::from("x"),
            &ExpressionEvalResult::IntegerResult(1));
        assert!(if let Ok(_) = assign_result { true } else { false} );
        let evaluation_result = w.eval(0, LineExecutionArgument::Empty, &mut ctxt);

        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext => true,
                _ => false
            });
    }

    #[test]
    fn it_skips_to_end_while_loop() {
        let mut ctxt = EvaluationContext::new(Box::new(DummyConsole{}));

        let the_while = GwWhile {
            condition: Box::new(GwVariableExpression { name: String::from("x") })
        };

        let abox:Box<dyn context::GwInstruction> = Box::new(the_while );
        let wend :Box<dyn context::GwInstruction> = Box::new(GwWend{});

        ctxt.real_lines = Some(vec![
            &abox,
            &wend
        ]);

        let assign_result =
            ctxt.set_variable(
                &String::from("x"),
                &ExpressionEvalResult::IntegerResult(0));
        assert!(if let Ok(_) = assign_result { true } else { false } );
        let evaluation_result = abox.eval(0, LineExecutionArgument::Empty, &mut ctxt);

        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLine(2) => true,
                _ => false
            });
    }
}
