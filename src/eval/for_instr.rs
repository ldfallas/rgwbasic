use super::{GwExpression, GwInstruction, EvaluationContext,
            LineExecutionArgument,
            InstructionResult, ExpressionEvalResult};

pub struct GwFor {
    pub variable: String,
    pub from: Box<dyn GwExpression>,
    pub to: Box<dyn GwExpression>,
    pub step: Option<Box<dyn GwExpression>>,
}

impl GwInstruction for GwFor {
    fn eval (&self,
             line: i16,
             arg: LineExecutionArgument,
             context: &mut EvaluationContext) -> InstructionResult {
        let mut next_line : i16 = 0;
        if let Some(corresponding_next) = context.pair_instruction_table.get(&line) {
            next_line = *corresponding_next;
        } else if let Some(ref real_lines) = context.real_lines {
            let index_of_next = find_next(line, real_lines);
            if index_of_next == -1 {
                return InstructionResult::EvaluateToError(String::from("NEXT WITHOUT FOR"));
            } else {
                context.pair_instruction_table.insert(line, index_of_next);
                context.pair_instruction_table.insert(index_of_next, line);
            }
            next_line = index_of_next;
        }

        if let LineExecutionArgument::NextIteration = arg {
            let variable_value = context.lookup_variable(&self.variable);
            if let Some(n_value) = get_as_integer(&variable_value) {
                let to_value = self.to.eval(context);
                if let Some(to_i_value) = get_as_integer(&Some(&to_value)) {
                    if to_i_value <= n_value {
                        InstructionResult::EvaluateLine(next_line + 1)
                    } else {
                        let new_value = ExpressionEvalResult::IntegerResult(n_value + 1);
                        context.set_variable(&self.variable, &new_value);
                        InstructionResult::EvaluateNext
                    }
                }
                else {
                    todo!();
                }

            } else{
                todo!();
            }

        } else {
            let result = self.from.eval(context);
            context.set_variable(&self.variable, &result);
            InstructionResult::EvaluateNext
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

fn find_next(line: i16, real_lines: &Vec<&Box<dyn GwInstruction>>) -> i16 {
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
             context: &mut EvaluationContext) -> InstructionResult {
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

fn get_as_integer(value: &Option<&ExpressionEvalResult>) -> Option<i16> {
    match value {
        Some(ExpressionEvalResult::IntegerResult(int_value)) =>  Some(*int_value),
        Some(ExpressionEvalResult::SingleResult(single_value)) => Some(*single_value as i16),
        Some(ExpressionEvalResult::DoubleResult(double_value)) =>  Some(*double_value as i16),
        _ => None
    }
}


#[cfg(test)]
mod for_eval_tests {
    use crate::eval::*;
    use crate::eval::for_instr::*;
    #[test]
    fn it_iteratates_for_loop() {
        let mut ctxt = EvaluationContext::new();
        let instr = GwFor {
            variable: String::from("x"),
            from: Box::new(GwIntegerLiteral{value: 1}),
            to: Box::new(GwIntegerLiteral{value: 3}),
            step: None
        };

        let abox:Box<dyn context::GwInstruction> = Box::new(instr );
        let inext :Box<dyn context::GwInstruction> = Box::new(GwNext{ variable: None});

        ctxt.real_lines = Some(vec![
            &abox,
            &inext
        ]);

        let mut tmp_arg = LineExecutionArgument::Empty;
        let mut evaluation_result = abox.eval(0, LineExecutionArgument::Empty, &mut ctxt);

        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext => true,
                _ => false
            });

        evaluation_result = inext.eval(1,tmp_arg, &mut ctxt);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
                    tmp_arg = tmp_arg2;
                    true
                },
                _ => false
            });

        evaluation_result = abox.eval(1,tmp_arg, &mut ctxt);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext => true,
                _ => false
            });

        evaluation_result = inext.eval(1,tmp_arg, &mut ctxt);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
                    tmp_arg = tmp_arg2;
                    true
                },
                _ => false
            });

        // evaluation_result = abox.eval(0,tmp_arg, &mut ctxt);

        // tmp_arg = LineExecutionArgument::Empty;
        // assert!(
        //     match evaluation_result {
        //         InstructionResult::EvaluateNext =>  true,
        //         _ => false
        //     });


        // evaluation_result = inext.eval(1,tmp_arg, &mut ctxt);
        // tmp_arg = LineExecutionArgument::Empty;
        // assert!(
        //     match evaluation_result {
        //         InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
        //             tmp_arg = tmp_arg2;
        //             true
        //         },
        //         _ => false
        //     });

        evaluation_result = abox.eval(0,tmp_arg, &mut ctxt);

        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateNext =>  true,
                _ => false
            });

        evaluation_result = inext.eval(1,tmp_arg, &mut ctxt);
        tmp_arg = LineExecutionArgument::Empty;
        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLineWithArg(0, tmp_arg2) => {
                    tmp_arg = tmp_arg2;
                    true
                },
                _ => false
            });


        evaluation_result = abox.eval(0,tmp_arg, &mut ctxt);

        assert!(
            match evaluation_result {
                InstructionResult::EvaluateLine(2) =>  true,
                _ => false
            });


    }
}
