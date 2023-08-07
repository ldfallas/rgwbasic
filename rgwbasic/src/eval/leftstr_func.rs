use super::{ GwExpression,
             EvaluationContext,
             ExpressionEvalResult,
             EvaluationError };

pub struct GwLeftStr {
    string_expr: Box<dyn GwExpression>,
    position_expr: Box<dyn GwExpression>
}

impl GwLeftStr {
    pub fn new(string_expr: Box<dyn GwExpression>,
           position_expr: Box<dyn GwExpression>) -> GwLeftStr {
        GwLeftStr {
            string_expr, position_expr
        }
    }
}

impl GwExpression for GwLeftStr {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        let binding = self.string_expr.eval(context)?;
        let string_value = binding.assume_string_value()?;
        let position_value = self.position_expr.eval(context)?.as_i16()?;

        if position_value >= 0 {
            Ok(ExpressionEvalResult::StringResult(
                string_value[0..(position_value as usize)].to_string()))
        } else {
            Err("Illegal function call".to_string())
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("LEFT$(");
        self.string_expr.fill_structure_string(buffer);
        buffer.push_str(", ");
        self.position_expr.fill_structure_string(buffer);
        buffer.push_str(")");
    }
}



#[cfg(test)]
mod left_str_tests {
    use crate::eval::{GwStringLiteral, GwIntegerLiteral};

    use super::*;
    use super::super::eval_tests::empty_context;
    
    #[test]
    fn it_extract_characters_inside_with_left() -> Result<(), String> {
        let left_call =
            GwLeftStr::new(
                Box::new(GwStringLiteral::with_value("APPLE".to_string())),
                Box::new(GwIntegerLiteral::with_value(3)));
        let mut ctxt = empty_context();
        let eval_result = left_call.eval(&mut ctxt)?;
        assert_eq!("APP", eval_result.assume_string_value()?);

        Ok(())
    }
}
