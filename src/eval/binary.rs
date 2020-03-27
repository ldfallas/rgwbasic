use crate::eval::GwExpression;
use crate::eval::EvaluationContext;
use crate::eval::ExpressionEvalResult;


pub enum GwBinaryOperationKind {
    Plus,
    Minus,
    Times,
    FloatDiv,
    IntDiv,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
    Equal,
    Different,
    Exponent,
    Mod,             
    And,
    Or,
    Eqv,
    Xor,
    Implication   
}

trait BinaryOperationEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16;
    fn perform_double_operation(&self, left : f32, right : f32) -> f32;
    
    fn evaluate(&self,
                left_result : &ExpressionEvalResult,
                right_result : &ExpressionEvalResult) -> ExpressionEvalResult {
        match (left_result, right_result) {
            (ExpressionEvalResult::IntegerResult(left),
             ExpressionEvalResult::IntegerResult(right)) =>
                ExpressionEvalResult::IntegerResult(self.perform_int_operation(*left, *right)),
            (ExpressionEvalResult::DoubleResult(left),
             ExpressionEvalResult::IntegerResult(right)) =>
                ExpressionEvalResult::DoubleResult(self.perform_double_operation(*left, f32::from(*right))),
            (ExpressionEvalResult::IntegerResult(left),
             ExpressionEvalResult::DoubleResult(right)) =>
                ExpressionEvalResult::DoubleResult(self.perform_double_operation(f32::from(*left), *right)),            
            (ExpressionEvalResult::DoubleResult(left),
             ExpressionEvalResult::DoubleResult(right)) =>
                ExpressionEvalResult::DoubleResult(self.perform_double_operation(*left, *right)),            
            (_, _) => panic!("Not implemented")
        }
    }
}

struct PlusEvaluator {
}

impl BinaryOperationEvaluator for PlusEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left + right
    }
    
    fn perform_double_operation(&self, left : f32, right : f32) -> f32 {
        left + right
    }
}

struct MinusEvaluator {
}

impl BinaryOperationEvaluator for MinusEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left - right
    }
    
    fn perform_double_operation(&self, left : f32, right : f32) -> f32 {
        left - right
    }
}

struct EqualEvaluator {
}

impl BinaryOperationEvaluator for EqualEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        if left == right {
            -1
        } else {
            0
        }
    }
    
    fn perform_double_operation(&self, left : f32, right : f32) -> f32 {
        if left == right {
            -1.0
        } else {
            0.0
        }
    }
}





struct TimesEvaluator {
}

impl BinaryOperationEvaluator for TimesEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left * right
    }
    
    fn perform_double_operation(&self, left : f32, right : f32) -> f32 {
        left * right           
    }
}

struct PowEvaluator {
}

impl BinaryOperationEvaluator for PowEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        let mut result = 1;

        for i in 1..right + 1  {
            result *= left;
        }
        result        
    }
    
    fn perform_double_operation(&self, left : f32, right : f32) -> f32 {
        left.powf(right)
    }
}

fn get_double_value(value : &ExpressionEvalResult) -> Option<f32> {
    match value {
        ExpressionEvalResult::DoubleResult(val) => Some(*val),
        ExpressionEvalResult::IntegerResult(intValue) => Some(f32::from(*intValue)),
        _ => None
    }
}

struct DivEvaluator {
}

impl BinaryOperationEvaluator for DivEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left / right
    }
    
    fn perform_double_operation(&self, left : f32, right : f32) -> f32 {
        left / right
    }

    fn evaluate(&self,
                left_result : &ExpressionEvalResult,
                right_result : &ExpressionEvalResult) -> ExpressionEvalResult {
        let left_double_value = get_double_value(left_result).unwrap();
        let right_double_value = get_double_value(right_result).unwrap();
        ExpressionEvalResult::DoubleResult(left_double_value / right_double_value)
    }

}


pub struct GwBinaryOperation {
    evaluator: Box<dyn BinaryOperationEvaluator>,
    kind: GwBinaryOperationKind,
    left: Box<dyn GwExpression>,
    right: Box<dyn GwExpression>
}



impl GwBinaryOperation {
    pub fn new(kind: GwBinaryOperationKind,
           left: Box<dyn GwExpression>,
           right: Box<dyn GwExpression>) -> GwBinaryOperation {
        let evaluator : Box<dyn BinaryOperationEvaluator> =  match kind {
            GwBinaryOperationKind::Plus => Box::new(PlusEvaluator {}),
            GwBinaryOperationKind::Minus => Box::new(MinusEvaluator {}),
            GwBinaryOperationKind::Times => Box::new(TimesEvaluator {}),
            GwBinaryOperationKind::FloatDiv => Box::new(DivEvaluator {}),
            GwBinaryOperationKind::Equal => Box::new(EqualEvaluator {}),
            GwBinaryOperationKind::Exponent => Box::new(PowEvaluator {}),                        
            _ => panic!("evaluator not implemented")
        };
        
        GwBinaryOperation {
            evaluator: evaluator,
            kind: kind,
            left: left,
            right: right
        }
    }
    
    fn fill_operator(&self, buffer : &mut String) {
        match self.kind {
            GwBinaryOperationKind::Plus => buffer.push_str(" + "),
            GwBinaryOperationKind::Times => buffer.push_str(" * "),
            GwBinaryOperationKind::Minus => buffer.push_str(" - "),
            GwBinaryOperationKind::FloatDiv => buffer.push_str(" / "),            
            _ => buffer.push_str(" ?? ")
        }
    }
}


impl GwExpression for GwBinaryOperation {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        self.evaluator.evaluate(&left_result, &right_result)
    }
    fn fill_structure_string(&self,   val : &mut String) {
        val.push_str("(");
        self.left.fill_structure_string(val);
        self.fill_operator(val);
        self.right.fill_structure_string(val);        
        val.push_str(")");
    }
}
