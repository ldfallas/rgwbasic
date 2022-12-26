use super::{ ExpressionType,
             ExpressionEvalResult,
             EvaluationContext,
             GwInstruction, GwAssignableExpression,
             LineExecutionArgument,
             InstructionResult };

/// AST element for DATA declaration
/// Example:
/// ```
///  DATA ABC,123,"adsf"
/// ```
pub struct GwData {
    // We store strings because data items could be used in different ways
    elements: Vec<String>
}


impl GwData {
    pub fn new(elements: Vec<String>) -> GwData {
        GwData { elements }
    }
}

impl GwInstruction for GwData {
    fn eval (&self,
             _line: i16,
             _argument: LineExecutionArgument,
             _context : &mut super::EvaluationContext) -> InstructionResult {
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("DATA ");
        let mut count = self.elements.len();
        for element in &self.elements {
            buffer.push_str(element);
            count -= 1;
            if count != 0 {
                buffer.push_str(", ");
            }
        }
    }

    fn get_data(&self) -> Option<&Vec<String>> { Some(&self.elements) }
}

/// AST element for reading DATA elements
/// Example:
/// ```
///  READ X
/// ```
pub struct GwRead {
    variable_expr: Box<dyn GwAssignableExpression>
}

impl GwRead {
    pub fn new(variable_expr: Box<dyn GwAssignableExpression>) -> GwRead {
        GwRead {
            variable_expr
        }
    }

    fn preprocess_numeric_data_item(&self, next_data: &String) -> Option<f64> {
        let without_suffix = next_data.trim_end_matches(|c| c == '#' || c == '$' || c == '%' || c == ' ');
        let without_prefix = without_suffix.trim_start();
        match without_prefix.parse::<f64>() {
            Ok(v) => Some(v),
            _ => None
        }
    }
    
    fn process_numeric_data_item_read<F>(&self,
                                         f: F,
                                         value_opt: Option<f64>,
                                         context: &mut EvaluationContext)
        -> InstructionResult
    where F: FnOnce(f64) -> ExpressionEvalResult {
        if let Some(value) = value_opt {
            check_result![
                self.variable_expr.assign_value(
                    f(value),
                    context)];
            InstructionResult::EvaluateNext
        } else {
            InstructionResult::EvaluateToError("Type mismatch".into())
        }
    }
}

impl GwInstruction for GwRead {
    fn eval (&self,
             _line: i16,
             _argument: LineExecutionArgument,
             context : &mut EvaluationContext) -> InstructionResult {
        let var_type = self.variable_expr.get_type(context);
        if let Some(next_data) = context.get_next_data_item() {
            match var_type {
                ExpressionType::String => {
                    let clonned_string = next_data.clone();
                    check_result![
                        self.variable_expr.assign_value(
                            ExpressionEvalResult::StringResult(clonned_string),
                            context)];
                    InstructionResult::EvaluateNext
                },
                ExpressionType::Integer => {
                    let value_opt = self.preprocess_numeric_data_item(next_data);
                    self.process_numeric_data_item_read(
                        |v| { ExpressionEvalResult::IntegerResult(v as i16) },
                        value_opt,
                        context)
                }
                ExpressionType::Single => {
                    let value_opt = self.preprocess_numeric_data_item(next_data);
                    self.process_numeric_data_item_read(
                        |v| { ExpressionEvalResult::SingleResult(v as f32) },
                        value_opt,
                        context)
                }
                ExpressionType::Double => {
                    let value_opt = self.preprocess_numeric_data_item(next_data);
                    self.process_numeric_data_item_read(
                        |v| { ExpressionEvalResult::DoubleResult(v) },
                        value_opt,
                        context)
                }
            }
        } else {
            InstructionResult::EvaluateToError("OUT OF DATA".to_string())
        }

    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("READ ");
        self.variable_expr.fill_structure_string(buffer);
    }
}

#[cfg(test)]
mod data_tests {
    use super::*;
    use crate::eval::{ ExpressionType, GwVariableExpression };

    #[test]
    fn it_reads_number_data() -> Result<(), & 'static str> {
        let string1 = "first".to_string();
        let string2 = "second".to_string();
        let mut ctx = EvaluationContext::new();
        ctx.set_variable_type("x", &ExpressionType::String);
        ctx.data = vec![&string1, &string2];
        let read_instr = GwRead::new(
            Box::new(GwVariableExpression::with_name("x".to_string())));
        match read_instr.eval(1, LineExecutionArgument::Empty, &mut ctx) {
            InstructionResult::EvaluateNext => {
                assert_eq!(ExpressionEvalResult::StringResult("first".to_string()),
                           *ctx.lookup_variable("x").unwrap());
                Ok(())
            },
            result => { println!("{:?}", result); Err("unexpected") }
        }

    }
}
