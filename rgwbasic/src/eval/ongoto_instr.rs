use super:: { GwInstruction, GwExpression, get_as_integer, InstructionResult };


pub struct GwOnGoto {
    expr: Box<dyn GwExpression>,
    cases: Vec<i16>
}

impl GwOnGoto {
    pub fn new( expr: Box<dyn GwExpression>,
                cases: Vec<i16>) -> GwOnGoto {
        GwOnGoto {
            expr,
            cases
        }
    }
}

impl GwInstruction for GwOnGoto {
    fn eval(&self,
            _line: i16,
            _argument: super::LineExecutionArgument,
            context: &mut super::EvaluationContext,
            _program: &mut super::GwProgram) -> super::InstructionResult {

        let expr_result = self.expr.eval(context);
        let evaluation: usize;
        match expr_result {
            Ok(ref eval) => {
                match get_as_integer(&Some(eval)) {
                    Ok(val) => {
                        evaluation = val as usize;
                    }
                    Err(error) => {
                        return InstructionResult::EvaluateToError(error);
                    }
                }

                if let Some(destination) = self.cases.get(evaluation - 1) {
                    calculate_jump_result(context, destination)
                }
                else {
                    InstructionResult::EvaluateNext
                }
            }
            Err(eval_error) => InstructionResult::EvaluateToError(eval_error)
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("ON ");
        self.expr.fill_structure_string(buffer);
        buffer.push_str(" GOTO ");
        let c = self.cases.len();
        let mut i = 0;
        for a_case in &self.cases {
            buffer.push_str(format!("{0}", a_case).as_str());
            if c != i + 1 {
                buffer.push_str(", ");
            }
            i += 1;
        }      

    }
}

fn calculate_jump_result(context: &mut super::EvaluationContext, destination: &i16) -> InstructionResult {
    if let Some(real_destination) = context.get_real_line(*destination) {
        InstructionResult::EvaluateLine(real_destination)
    } else {
        InstructionResult::EvaluateToError(String::from("Cannot find real line"))
    }
}


#[cfg(test)]
mod on_goto_tests {
    use crate::eval::eval_tests::DummyConsole;
    use crate::eval::*;
    use crate::eval::ongoto_instr::*;
    use std::rc::Rc;
   

    #[test]
    fn it_performs_simple_on_goto() -> Result<(),& 'static  str> {
        let mut ctxt = EvaluationContext::new(Box::new(DummyConsole{}));
        ctxt.jump_table.insert(10, 11);
        ctxt.jump_table.insert(20, 21);
        let on_goto = Rc::new(GwOnGoto {
            expr: Box::new(GwVariableExpression { name: String::from("x") }),
            cases: vec![10,20,30]
        });


        let mut program = GwProgram {
            lines: vec![],
            real_lines: vec![on_goto.clone()],
            data: vec![],
        };

        let _ = ctxt.set_variable(
            &String::from("x"),
            &ExpressionEvalResult::IntegerResult(2));

        match on_goto.eval(1,
                           LineExecutionArgument::Empty,
                           &mut ctxt,
                           &mut program) {
            InstructionResult::EvaluateLine(21) => Ok(()),
            _ => Err("Unexpected state in ON GOTO")
        }
    }


    #[test]
    fn it_performs_fallthrough_on_goto() -> Result<(),& 'static  str> {
        let mut ctxt = EvaluationContext::new(Box::new(DummyConsole{}));
        let on_goto = Rc::new(GwOnGoto {
            expr: Box::new(GwVariableExpression { name: String::from("x") }),
            cases: vec![10,20,30]
        });


        let mut program = GwProgram {
            lines: vec![],
            real_lines: vec![on_goto.clone()],
            data: vec![],
        };

        let _ = ctxt.set_variable(
            &String::from("x"),
            &ExpressionEvalResult::IntegerResult(23));

        match on_goto.eval(1,
                           LineExecutionArgument::Empty,
                           &mut ctxt,
                           &mut program) {
            InstructionResult::EvaluateNext => Ok(()),
            _ => Err("Unexpected state in ON GOTO")
        }
    }    
}
