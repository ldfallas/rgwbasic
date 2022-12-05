use super::context::{EvaluationContext, ExpressionType, GwInstruction,
                     InstructionResult, LineExecutionArgument};

pub enum DefVarRange {
    Single(char),
    Range(char, char)
}

pub struct GwDefType {
    ranges : Vec<DefVarRange>,
    definition_type: ExpressionType
}

impl GwDefType {
    pub fn with_var_range(var_range : Vec<DefVarRange>,
                          definition_type: ExpressionType)
                          -> GwDefType {
        GwDefType { ranges : var_range,
                    definition_type
        }
    }
}

impl GwInstruction for GwDefType {
    fn eval (&self,
             _line: i16,
             _arg: LineExecutionArgument,
             context : &mut EvaluationContext) -> InstructionResult{

        for range in &self.ranges {
            set_range_type(&range, &self.definition_type, context);
        }
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        let instruction_name =
        match self.definition_type {
            ExpressionType::Double => "DEFDBL",
            ExpressionType::Integer => "DEFINT",
            ExpressionType::String => "DEFSTR",
            ExpressionType::Single => "DEFSNG",
        };
        buffer.push_str(instruction_name);
        buffer.push_str(&" ");
        range_to_string_buffer(&self.ranges, buffer);
    }
}

fn range_to_string_buffer(ranges: &Vec<DefVarRange>, buffer: &mut String) {
    for obj in ranges {
        match obj {
            DefVarRange::Single(c) =>
                buffer.push_str(&c.to_string()[..]),
            DefVarRange::Range(s, e) => {
                buffer.push_str(&s.to_string()[..]);
                buffer.push_str("-");
                buffer.push_str(&e.to_string()[..]);
            }
        }
        buffer.push_str(",");
    }
}

fn set_range_type(range: &DefVarRange,
                  range_type: &ExpressionType,
                  context: &mut EvaluationContext) {
    let mut tmp_str = String::with_capacity(1);
    match range {
        DefVarRange::Single(var_name) => {
            tmp_str.push(*var_name);
            context.set_variable_type(&tmp_str, range_type);
        }
        DefVarRange::Range(first_char, second_char) => {
            if first_char.is_ascii_alphabetic() {
                let mut tmp_char = *first_char as u8;
                let end_char = *second_char as u8;
                while tmp_char < end_char {
                    tmp_str.clear();
                    tmp_str.push(tmp_char as char);
                    context.set_variable_type(&tmp_str, range_type);
                    tmp_char += 1;
                }
            }
        }
    }
}


#[cfg(test)]
mod definition_type_tests {
    use super::*;
    use crate::eval::*;
    use crate::eval::eval_tests::empty_context;

    #[test]
    fn it_sets_double_type() -> Result<(), & 'static str> {

        assert_eq!((true, true, false, true),
                   try_to_assign("x", ExpressionType::Double));
        Ok(())
    }

    #[test]
    fn it_sets_int_type() -> Result<(), & 'static str> {

        assert_eq!((true, true, false, true),
                   try_to_assign("x", ExpressionType::Integer));
        Ok(())
    }
    
    #[test]
    fn it_sets_single_type() -> Result<(), & 'static str> {

        assert_eq!((true, true, false, true),
                   try_to_assign("x", ExpressionType::Single));
        Ok(())
    }

    #[test]
    fn it_sets_string_type() -> Result<(), & 'static str> {

        assert_eq!((false, false, true, false),
                   try_to_assign("x", ExpressionType::String));
        Ok(())
    }

    
    fn try_to_assign(var_name: &str, declare_as: ExpressionType)
                     -> (bool,bool,bool,bool) {
        let var_name_char = var_name.chars().nth(0).unwrap();
        let instr = GwDefType::with_var_range(vec![DefVarRange::Single(var_name_char)],
                                              declare_as);
        let mut context = empty_context();

        instr.eval(1, LineExecutionArgument::Empty, &mut context);
        
        let dbl_assign = context.set_variable(var_name, &ExpressionEvalResult::DoubleResult(3 as f64));
        let single_assign = context.set_variable(var_name, &ExpressionEvalResult::SingleResult(3 as f32));
        let string_assign = context.set_variable(var_name, &ExpressionEvalResult::StringResult("xa".to_string()));
        let int_assign = context.set_variable(var_name, &ExpressionEvalResult::IntegerResult(0));
                                                
        
        (dbl_assign.is_ok(),
         single_assign.is_ok(),
         string_assign.is_ok(),
         int_assign.is_ok())
    }
}
