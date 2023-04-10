use std::rc::Rc;

use super::{GwExpression, GwInstruction, GwProgram, EvaluationContext,
            LineExecutionArgument,
            InstructionResult, ExpressionEvalResult, evaluate_to_usize};

pub struct GwFor {
    pub variable: String,
    pub from: Box<dyn GwExpression>,
    pub to: Box<dyn GwExpression>,
    pub step: Option<Box<dyn GwExpression>>,
}

impl GwFor {
    fn try_next_iteration(&self,
                          next_line : i16,
                          context: &mut EvaluationContext)
                          -> Result<InstructionResult, String> {
        let n_value = get_as_integer(& context.lookup_variable(&self.variable))? as usize;
        let to_i_value = evaluate_to_usize(&self.to, context)?;
        if to_i_value <= n_value {
            Ok(InstructionResult::EvaluateLine(next_line + 1))
        } else {
            let new_value = ExpressionEvalResult::IntegerResult((n_value as i16) + 1);
            context.set_variable(&self.variable, &new_value)?;
            Ok(InstructionResult::EvaluateNext)
        }
    }
}

impl GwInstruction for GwFor {
    fn eval (&self,
             line: i16,
             arg: LineExecutionArgument,
             context: &mut EvaluationContext,
             program: &mut GwProgram) -> InstructionResult {
        //context.console.log(format!(">>> {:?}", arg).as_str());

        let mut next_line : i16 = 0;
        if let Some(corresponding_next) = context.pair_instruction_table.get(&line) {
            next_line = *corresponding_next;
        } else { //if let Some(ref real_lines) = context.real_lines {
            let index_of_next = find_next(line, &program.real_lines);
            if index_of_next == -1 {
                return InstructionResult::EvaluateToError(String::from("NEXT WITHOUT FOR"));
            } else {
                context.pair_instruction_table.insert(line, index_of_next);
                context.pair_instruction_table.insert(index_of_next, line);
            }
            next_line = index_of_next;
        }

        if let LineExecutionArgument::NextIteration = arg {
            match self.try_next_iteration(next_line, context) {
                Ok(result) => result,
                Err(err) => InstructionResult::EvaluateToError(err)
            }
        } else {
            if let Ok(result) = self.from.eval(context) {
                let assign_result = context.set_variable(&self.variable, &result);
                if assign_result.is_ok() {
                    InstructionResult::EvaluateNext
                } else {
                    InstructionResult::EvaluateToError(assign_result.err().unwrap().to_string())
                }
            } else {
                todo!();
            }
        }
    }

    fn is_for(&self) -> bool { true }

//    fn get_for_info(&self) -> Option<&GwFor> {
//        Some(self)
//    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"FOR");
    }
}

fn find_next(line: i16, real_lines: &Vec<Rc<dyn GwInstruction>>) -> i16 {
    let mut curr_line = line + 1;
    let mut next_balance = 0;
    loop {
        if curr_line >= real_lines.len() as i16 {
            break;
        } else if let Some(ref instr) = real_lines.get(curr_line as usize) {

            if instr.is_for() {
                next_balance += 1;
            }
            if instr.is_next() {
                if next_balance  == 0 {
                    return curr_line as i16;
                } else {
                    next_balance -= 1;
                }
            }
        }
        curr_line += 1;
    }
    return -1;
}




pub struct GwNext {
    pub variable: Option<String>
}

impl GwInstruction for GwNext {

    fn eval (&self,
             line: i16,
             _arg: LineExecutionArgument,
             context: &mut EvaluationContext,
             program: &mut GwProgram) -> InstructionResult {
        if let Some(corresponding_for) =  context.pair_instruction_table.get(&line) {
            InstructionResult::EvaluateLineWithArg(
                *corresponding_for,
                LineExecutionArgument::NextIteration)
        } else {
            InstructionResult::EvaluateToError(String::from("NEXT WITHOUT FOR"))
        }
    }

    fn is_next(&self) -> bool { true }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"NEXT");
    }
}

fn get_as_integer(value: &Option<&ExpressionEvalResult>) -> Result<i16, String> {
    match value {
        Some(ExpressionEvalResult::IntegerResult(int_value)) =>  Ok(*int_value),
        Some(ExpressionEvalResult::SingleResult(single_value)) => Ok(*single_value as i16),
        Some(ExpressionEvalResult::DoubleResult(double_value)) =>  Ok(*double_value as i16),
        _ => Err("Type mismatch".to_string())
    }
}


#[cfg(test)]
mod for_eval_tests {
    use crate::eval::*;
    use crate::eval::for_instr::*;
    use crate::eval::eval_tests::{ DummyConsole, empty_program };
    
    #[test]
    fn it_iteratates_for_loop() {
        let mut ctxt = EvaluationContext::new(Box::new(DummyConsole{}));
        let instr = GwFor {
            variable: String::from("x"),
            from: Box::new(GwIntegerLiteral{value: 1}),
            to: Box::new(GwIntegerLiteral{value: 3}),
            step: None
        };

        let abox: Rc<dyn context::GwInstruction> = Rc::new(instr );
        let inext: Rc<dyn context::GwInstruction> = Rc::new(GwNext{ variable: None});

        let mut program = GwProgram {            
            real_lines: vec![
                abox.clone(),
                inext.clone()
            ],
            data: vec![],
            lines: vec![]
        };

        let mut tmp_arg = LineExecutionArgument::Empty;
        let mut evaluation_result = abox.eval(0,
                                              LineExecutionArgument::Empty,
                                              &mut ctxt,
                                              &mut program);

        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext => true,
                _ => false
            });

        evaluation_result = inext.eval(1,tmp_arg, &mut ctxt, &mut program);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
                    tmp_arg = tmp_arg2;
                    true
                },
                _ => false
            });

        evaluation_result = abox.eval(1,tmp_arg, &mut ctxt, &mut program);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext => true,
                _ => false
            });

        evaluation_result = inext.eval(1,tmp_arg, &mut ctxt, &mut program);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
                    tmp_arg = tmp_arg2;
                    true
                },
                _ => false
            });

        evaluation_result = abox.eval(0,tmp_arg, &mut ctxt, &mut program);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext =>  true,
                _ => false
            });

        evaluation_result = inext.eval(1,tmp_arg, &mut ctxt, &mut program);
        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
                    tmp_arg = tmp_arg2;
                    true
                },
                _ => false
            });


        evaluation_result = abox.eval(0,tmp_arg, &mut ctxt, &mut program);

        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLine(2) =>  true,
                _ => false
            });
    }
}
