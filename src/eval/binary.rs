use crate::eval::GwExpression;
use crate::eval::EvaluationContext;
use crate::eval::ExpressionEvalResult;
use crate::eval::EvaluationError;


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
    fn perform_int_operation(&self, left: i16, right: i16) -> i16;
    fn perform_single_operation(&self, left: f32, right: f32) -> f32;
    fn perform_double_operation(&self, left: f64, right: f64) -> f64;
    fn perform_string_operation(&self, _left: &String, _right: &String)
                                -> Result<ExpressionEvalResult, &'static str> {
        Err("Type mismatch")
    }

    fn evaluate(&self,
                left_result : &ExpressionEvalResult,
                right_result : &ExpressionEvalResult)
                -> Result<ExpressionEvalResult, &'static str> {
        match (left_result, right_result) {
            (ExpressionEvalResult::IntegerResult(left), right) => self.evaluate_int_vs(*left, right),
            (ExpressionEvalResult::SingleResult(left), right) =>  self.evaluate_single_vs(*left, right),            
            (ExpressionEvalResult::DoubleResult(left), right) =>  self.evaluate_double_vs(*left, right),
            (ExpressionEvalResult::StringResult(left),
             ExpressionEvalResult::StringResult(right)) =>
                self.perform_string_operation(left, right),
            (_, _) => Err("Type mismatch")
        }
    }

    fn evaluate_int_vs(&self,
                left: i16,
                right_result: &ExpressionEvalResult)
                -> Result<ExpressionEvalResult, &'static str> {
        match right_result {
             ExpressionEvalResult::IntegerResult(right) =>
                Ok(ExpressionEvalResult::IntegerResult(self.perform_int_operation(left, *right))),
            ExpressionEvalResult::DoubleResult(right) =>
                Ok(ExpressionEvalResult::DoubleResult(self.perform_double_operation(f64::from(left), *right))),
            ExpressionEvalResult::SingleResult(right) =>
                Ok(ExpressionEvalResult::SingleResult(self.perform_single_operation(f32::from(left), *right))),
            _  => Err("Type mismatch")
        }
    }

    fn evaluate_single_vs(&self,
                          left: f32,
                          right_result : &ExpressionEvalResult)
                -> Result<ExpressionEvalResult, &'static str> {
        match right_result {
             ExpressionEvalResult::IntegerResult(right) =>
                Ok(ExpressionEvalResult::SingleResult(self.perform_single_operation(left, f32::from(*right)))),
            ExpressionEvalResult::DoubleResult(right) =>
                Ok(ExpressionEvalResult::DoubleResult(self.perform_double_operation(f64::from(left), *right))),
            ExpressionEvalResult::SingleResult(right) =>
                Ok(ExpressionEvalResult::SingleResult(self.perform_single_operation(left, *right))),
            _  => Err("Type mismatch")
        }
    }

    fn evaluate_double_vs(&self,
                          left: f64,
                          right_result : &ExpressionEvalResult)
                -> Result<ExpressionEvalResult, &'static str> {
        match right_result {
             ExpressionEvalResult::IntegerResult(right) =>
                Ok(ExpressionEvalResult::DoubleResult(self.perform_double_operation(left, f64::from(*right)))),
            ExpressionEvalResult::DoubleResult(right) =>
                Ok(ExpressionEvalResult::DoubleResult(self.perform_double_operation(f64::from(left), *right))),
            ExpressionEvalResult::SingleResult(right) =>
                Ok(ExpressionEvalResult::DoubleResult(self.perform_double_operation(left , f64::from(*right)))),
            _  => Err("Type mismatch")
        }
    }
}

struct PlusEvaluator {
}

impl BinaryOperationEvaluator for PlusEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left + right
    }

    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        left + right
    }
    
    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
        left + right
    }
}

struct MinusEvaluator {
}

impl BinaryOperationEvaluator for MinusEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left - right
    }

    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        left - right
    }

    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
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

    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        if left == right {
            -1.0
        } else {
            0.0
        }
    }
    
    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
        if left == right {
            -1.0
        } else {
            0.0
        }
    }
}


struct DifferentEvaluator {
}

impl BinaryOperationEvaluator for DifferentEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        if left != right {
            -1
        } else {
            0
        }
    }

    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        if left != right {
            -1.0
        } else {
            0.0
        }
    }

    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
        if left != right {
            -1.0
        } else {
            0.0
        }
    }
}

struct LessThanEvaluator {
}

impl BinaryOperationEvaluator for LessThanEvaluator {
    fn perform_int_operation(&self, left: i16, right: i16) -> i16 {
        bool_to_basic_value(left < right)
    }

    fn perform_single_operation(&self, left: f32, right: f32) -> f32 {
        bool_to_basic_value(left < right) as f32
    }
    
    fn perform_double_operation(&self, left: f64, right: f64) -> f64 {
        bool_to_basic_value(left < right) as f64
    }

    fn perform_string_operation(&self, left: &String, right: &String)
                                -> Result<ExpressionEvalResult, &'static str> {
        Ok(ExpressionEvalResult::IntegerResult(
            bool_to_basic_value(left.cmp(right).is_lt())))
    }
}

struct GreaterThanEvaluator {
}

impl BinaryOperationEvaluator for GreaterThanEvaluator {
    fn perform_int_operation(&self, left: i16, right: i16) -> i16 {
        bool_to_basic_value(left > right)
    }

    fn perform_single_operation(&self, left: f32, right: f32) -> f32 {
        bool_to_basic_value(left > right) as f32
    }
    
    fn perform_double_operation(&self, left: f64, right: f64) -> f64 {
        bool_to_basic_value(left > right) as f64
    }

    fn perform_string_operation(&self, left: &String, right: &String)
                                -> Result<ExpressionEvalResult, &'static str> {
        Ok(ExpressionEvalResult::IntegerResult(
            bool_to_basic_value(left.cmp(right).is_gt())))
    }
}


struct LessEqualThanEvaluator {
}

impl BinaryOperationEvaluator for LessEqualThanEvaluator {
    fn perform_int_operation(&self, left: i16, right: i16) -> i16 {
        bool_to_basic_value(left <= right)
    }

    fn perform_double_operation(&self, left: f64, right: f64) -> f64 {
        bool_to_basic_value(left <= right) as f64
    }

    fn perform_single_operation(&self, left: f32, right: f32) -> f32 {
        bool_to_basic_value(left <= right) as f32
    }

    fn perform_string_operation(&self, left: &String, right: &String)
                                -> Result<ExpressionEvalResult, &'static str> {
        Ok(ExpressionEvalResult::IntegerResult(
            bool_to_basic_value(left.cmp(right).is_le())))
    }
}


struct GreaterEqualThanEvaluator {
}

impl BinaryOperationEvaluator for GreaterEqualThanEvaluator {
    fn perform_int_operation(&self, left: i16, right: i16) -> i16 {
        bool_to_basic_value(left >= right)
    }

    fn perform_double_operation(&self, left: f64, right: f64) -> f64 {
        bool_to_basic_value(left >= right) as f64
    }

    fn perform_single_operation(&self, left: f32, right: f32) -> f32 {
        bool_to_basic_value(left >= right) as f32
    }

    fn perform_string_operation(&self, left: &String, right: &String)
                                -> Result<ExpressionEvalResult, &'static str> {
        Ok(ExpressionEvalResult::IntegerResult(
            bool_to_basic_value(left.cmp(right).is_ge())))
    }
}

struct TimesEvaluator {
}

impl BinaryOperationEvaluator for TimesEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left * right
    }

    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
        left * right
    }
    
    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        left * right
    }
}

struct PowEvaluator {
}

impl BinaryOperationEvaluator for PowEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        let mut result = 1;

        for _i in 1..right + 1  {
            result *= left;
        }
        result
    }

    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
        left.powf(right)
    }

    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        left.powf(right)
    }
}

fn get_double_value(value : &ExpressionEvalResult) -> Option<f64> {
    match value {
        ExpressionEvalResult::DoubleResult(val) => Some(*val),
        ExpressionEvalResult::IntegerResult(int_value) => Some(f64::from(*int_value)),
        &ExpressionEvalResult::SingleResult(int_value) => Some(int_value as f64),
        _ => None
    }
}

struct DivEvaluator {
}

impl BinaryOperationEvaluator for DivEvaluator {
    fn perform_int_operation(&self, left : i16, right : i16) -> i16 {
        left / right
    }

    fn perform_single_operation(&self, left : f32, right : f32) -> f32 {
        left / right
    }
    
    fn perform_double_operation(&self, left : f64, right : f64) -> f64 {
        left / right
    }

    fn evaluate(&self,
                left_result : &ExpressionEvalResult,
                right_result : &ExpressionEvalResult) -> Result<ExpressionEvalResult, &'static str> {
        let left_double_value = get_double_value(left_result).unwrap();
        let right_double_value = get_double_value(right_result).unwrap();
        Ok(ExpressionEvalResult::DoubleResult(left_double_value / right_double_value))
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
            GwBinaryOperationKind::Different => Box::new(DifferentEvaluator {}),
            GwBinaryOperationKind::Exponent => Box::new(PowEvaluator {}),
            GwBinaryOperationKind::GreaterThan => Box::new(GreaterThanEvaluator {}),
            GwBinaryOperationKind::LessThan => Box::new(LessThanEvaluator {}),
            GwBinaryOperationKind::GreaterEqualThan => Box::new(GreaterEqualThanEvaluator {}),
            GwBinaryOperationKind::LessEqualThan => Box::new(LessEqualThanEvaluator {}),
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
	    GwBinaryOperationKind::Equal => buffer.push_str(" = "),
            GwBinaryOperationKind::FloatDiv => buffer.push_str(" / "),
            GwBinaryOperationKind::Different => buffer.push_str(" <> "),
            GwBinaryOperationKind::Exponent => buffer.push_str(" ^ "),
            GwBinaryOperationKind::GreaterThan => buffer.push_str(" > "),
            GwBinaryOperationKind::LessThan => buffer.push_str(" < "),
            GwBinaryOperationKind::GreaterEqualThan => buffer.push_str(" >= "),
            GwBinaryOperationKind::LessEqualThan => buffer.push_str(" <= "),
            _ => buffer.push_str(" ?? ")
        }
    }
}


impl GwExpression for GwBinaryOperation {
    fn eval (&self, context : &mut EvaluationContext) 
                      -> Result<ExpressionEvalResult, EvaluationError> {
        match (self.left.eval(context), self.right.eval(context)) {
            (Ok(left_result), Ok(right_result)) => {
               match self.evaluator.evaluate(&left_result, &right_result) {
                   Ok(result) => Ok(result),
                   Err(err) => Err(err.to_string())
               }
            }
            _ => Err("Error on binary operation".to_string())

        }
    }
    fn fill_structure_string(&self,   val : &mut String) {
        val.push_str("(");
        self.left.fill_structure_string(val);
        self.fill_operator(val);
        self.right.fill_structure_string(val);
        val.push_str(")");
    }
}

#[inline(always)]
fn bool_to_basic_value(flag: bool) -> i16 {
    if flag {
        -1
    } else {
        0
    }
}
