pub mod context;
pub mod binary;
pub mod while_instr;

//use std::fs::File;
//use std::io::BufReader;
use std::io;
use std::io::prelude::*;
use std::process::exit;
//use crate::parser::ParserResult;

pub use crate::eval::context::{EvaluationContext,
			       ExpressionEvalResult,
			       GwVariableType,
			       GwInstruction,
			       GwProgram,
			       ProgramLine,
			       InstructionResult};




pub trait GwExpression {
    fn eval(&self, context : &mut EvaluationContext) -> ExpressionEvalResult ;
    fn fill_structure_string(&self,   buffer : &mut String);
}

pub trait GwAssignableExpression : GwExpression {
    fn get_type(&self, context : &EvaluationContext) -> GwVariableType;
    fn assign_value(&self, value : ExpressionEvalResult, context : &mut EvaluationContext);
}

pub struct GwParenthesizedAccessExpr {
    name : String,
    arguments : Vec<Box<dyn GwExpression>>
}

// impl GwParenthesizedAccessExpr {
//     fn new(name : String, arguments : Vec<Box<dyn GwExpression>>)
//            -> GwParenthesizedAccessExpr

//     {
//         GwParenthesizedAccessExprx {
//             name,
//             arguments
//         }
//     }    
// }

fn try_to_get_integer_index(eval_result : &ExpressionEvalResult) -> Option<u16> {
    match eval_result {
        ExpressionEvalResult::IntegerResult(i_result) => Some(*i_result as u16),
        ExpressionEvalResult::DoubleResult(d_result) => Some(*d_result as u16),
        _ => None
    }
}

impl GwExpression for GwParenthesizedAccessExpr {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        let mut evaluated_arguments = Vec::with_capacity(self.arguments.len());
        for arg in &self.arguments {
            let eval_index = arg.eval(context);
            evaluated_arguments.push(try_to_get_integer_index(&eval_index).unwrap());
        }
        
        if let Some(array) = context.get_existing_array(
            &self.name,
            self.arguments.len()) {
            return array.get_value(evaluated_arguments);
        } else if let Some(_function) = context.get_existing_function(
            &self.name,
            self.arguments.len()) {
            panic!("Not implemented");
        } else {
            panic!("Cannot use parenthesis with identifier {}", self.name);
        }        
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.name[..]);
        buffer.push_str("(");
        buffer.push_str(")");        
    }

}

pub struct GwCall {
    pub array_or_function : String,
    pub arguments : Vec<Box<dyn GwExpression>>
}

impl GwAssignableExpression for GwCall {
    fn get_type(&self, context : &EvaluationContext) -> GwVariableType {
	return context.get_variable_type(&self.array_or_function).unwrap();
    }
    
    fn assign_value(&self,
		    _value : ExpressionEvalResult,
		    _context : &mut EvaluationContext) {
	panic!("AHHHHH");
    }
}

impl GwExpression for GwCall {
    fn eval (&self, _context : &mut EvaluationContext) -> ExpressionEvalResult {
        panic!("Not implemented");
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.array_or_function[..]);
        buffer.push_str("(");
        let mut i = 0;
        for arg in &self.arguments {
            arg.fill_structure_string(buffer);
            if i != &self.arguments.len() - 1 {
                buffer.push(',')
            }
            i = i + 1
        }
        buffer.push_str(")");        
    }    
}

pub struct GwLog {
    pub expr: Box<dyn GwExpression>
}

impl GwExpression for GwLog {
    fn eval(&self, context: &mut EvaluationContext) -> ExpressionEvalResult {
        match self.expr.eval(context) {
            ExpressionEvalResult::IntegerResult(value) => 
                ExpressionEvalResult::DoubleResult((value as f32).ln()),            
            ExpressionEvalResult::DoubleResult(value) =>
                ExpressionEvalResult::DoubleResult(value.ln()),
            _ => {panic!("incorrect type")}
        }
    }
    fn fill_structure_string(&self, buffer : &mut String) {        
        buffer.push_str("LOG(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }  
}

pub struct GwInt {
    pub expr: Box<dyn GwExpression>
}

impl GwExpression for GwInt {
    fn eval(&self, context: &mut EvaluationContext) -> ExpressionEvalResult {
        match self.expr.eval(context) {
            full@ExpressionEvalResult::IntegerResult(_) =>
                full,
            ExpressionEvalResult::DoubleResult(value) =>
                ExpressionEvalResult::IntegerResult(value as i16), 
            _ => {panic!("incorrect type")}
        }
    }
    fn fill_structure_string(&self, buffer : &mut String) {        
        buffer.push_str("INT(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }  
}


pub struct GwNegExpr {
    pub expr: Box<dyn GwExpression>
}

impl GwExpression for GwNegExpr {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        match self.expr.eval(context) {
            ExpressionEvalResult::IntegerResult(value) =>
                ExpressionEvalResult::IntegerResult(-1 * value),
            ExpressionEvalResult::DoubleResult(value) =>
                ExpressionEvalResult::DoubleResult(-1.0 * value),            
            _ => {panic!("incorrect type")}
        }
    }
    fn fill_structure_string(&self, buffer : &mut String) {        
        buffer.push_str("-");
        self.expr.fill_structure_string(buffer);
    }
}


pub struct GwParenthesizedExpr {
    expr: Box<dyn GwExpression>
}

impl GwParenthesizedExpr {
    pub fn new(expr : Box<dyn GwExpression>)
               -> GwParenthesizedExpr {
        GwParenthesizedExpr {
            expr: expr
        }
    }    
}

impl GwExpression for GwParenthesizedExpr {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        return self.expr.eval(context);
    }
    fn fill_structure_string(&self, buffer : &mut String) {        
        buffer.push_str("(");
        self.expr.fill_structure_string(buffer);
        buffer.push_str(")");        
    }
}

pub struct GwStringLiteral {
    value : String
}

impl GwStringLiteral {
    pub fn with_value(value : String) -> GwStringLiteral {
        GwStringLiteral { value } 
    }

}

impl GwExpression for GwStringLiteral {
    fn eval (&self, _context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::StringResult(String::from(&self.value))
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.value[..]);
    }
}

pub struct GwIntegerLiteral {
    value : i16
}

impl GwIntegerLiteral {
    pub fn with_value(value : i16) -> GwIntegerLiteral {
        GwIntegerLiteral { value }
    }
}

impl GwExpression for GwIntegerLiteral {
    fn eval (&self, _context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::IntegerResult(self.value)
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.value.to_string());
    }    
}

pub struct GwDoubleLiteral {
    value : f32
}

impl GwDoubleLiteral {
    pub fn with_value(value : f32) -> GwDoubleLiteral {
        GwDoubleLiteral { value }
    }
}

impl GwExpression for GwDoubleLiteral {
    fn eval (&self, _context : &mut EvaluationContext) -> ExpressionEvalResult {
        ExpressionEvalResult::DoubleResult(self.value)
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.value.to_string());
    }    
}



pub struct GwVariableExpression {
    name : String
}

impl GwAssignableExpression for GwVariableExpression {
    fn get_type(&self, context : &EvaluationContext) -> GwVariableType {
	 context.get_variable_type(&self.name).unwrap_or(GwVariableType::Double)
    }
    
    fn assign_value(&self, value : ExpressionEvalResult, context : &mut EvaluationContext) {
        context.set_variable(&self.name, &value);
    }
}

impl GwVariableExpression {
    pub fn with_name(name : String) -> GwVariableExpression {
        GwVariableExpression { name }
    }
}

impl GwExpression for GwVariableExpression {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        if let Some(value) =  context.lookup_variable(&self.name) {
            value.clone()
        } else {
            // TODO we need to define a variable here???
            ExpressionEvalResult::IntegerResult(0)
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.name[..]);
    }
}


impl ProgramLine {
    fn get_line(&self) -> i16 {
        self.line
    }
    
    // fn eval (&self, context : &mut EvaluationContext) -> InstructionResult {
    //      self.instruction.eval(self.line, context)
    // }

    pub fn fill_structure_string(&self,   buffer : &mut String) {
        buffer.push('(');
        buffer.push_str(&self.line.to_string()[..]);
        buffer.push(' ');
        self.instruction.fill_structure_string(buffer);
        if let Some(rest) = &self.rest_instructions {
            buffer.push(' ');
            for e in rest {
                buffer.push(':');
                e.fill_structure_string(buffer);
            }
        }
        buffer.push(')');        
    }
}

pub struct GwListStat {
}

impl GwInstruction for GwListStat {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        if let Some(up) =  &context.underlying_program {
            up.list();
        }
        
        InstructionResult::EvaluateNext
    }    
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"LIST");
    }
}


pub struct GwLoadStat {
    pub filename : Box<dyn GwExpression>
}

impl GwInstruction for GwLoadStat {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        let result = self.filename.eval(context);
        if let Some(up) =  &mut context.underlying_program {
            match result {
                ExpressionEvalResult::StringResult(filename) =>
                {
                    match up.load_from(&filename.trim_matches('"')) {
                        Ok(_) => {
                            println!("File loaded");
                        }
                        Err(error) => {
                            panic!("Error loading file {:?}", error);
                        }
                    }
                },
                _ => {panic!("Type mismatch"); }
            }

        }       
        InstructionResult::EvaluateNext
    }    
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"LOAD ");
        self.filename.fill_structure_string(buffer);

    }
}

pub struct GwRunStat {
}

impl GwInstruction for GwRunStat {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        if let Some(program) = &context.underlying_program {
            program.run();
        }
        InstructionResult::EvaluateNext
    }    
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"RUN");
    }
}

pub struct GwSystemStat {
}

impl GwInstruction for GwSystemStat {
    fn eval (&self, _line: i16, _context : &mut EvaluationContext) -> InstructionResult{
        exit(0);
    }    
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"RUN");
    }
}

pub enum DefVarRange {
    Single(char),
    Range(char, char) 
}

pub struct GwDefDbl {
    ranges : Vec<DefVarRange>
}

impl GwDefDbl {
    pub fn with_var_range(var_range : Vec<DefVarRange>)
                          -> GwDefDbl {
        GwDefDbl { ranges : var_range }
    }
}

impl GwInstruction for GwDefDbl {
    fn eval (&self, _line: i16, _context : &mut EvaluationContext) -> InstructionResult{
        // TODO implementation pending
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"DEFDBL ");
        for obj in &self.ranges {
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
}

pub struct GwIf {
    condition : Box<dyn GwExpression>,
    then_line : i16
}

impl GwIf {
    pub fn new(condition : Box<dyn GwExpression>, then_line : i16) -> GwIf {
        return GwIf {
            condition,
            then_line
        };
    }
}

impl GwInstruction for GwIf {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        match self.condition.eval(context) {
            ExpressionEvalResult::IntegerResult(i_result) if i_result == 0 => InstructionResult::EvaluateNext,
            ExpressionEvalResult::DoubleResult(d_result) if d_result == 0.0 => InstructionResult::EvaluateNext,
            _ => {
		if let Some(real_line) = context.get_real_line(self.then_line) {
		    InstructionResult::EvaluateLine(real_line)
		} else {
		    panic!("Jumping to a non-existing line!");
		}
	    }
        }        
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"IF ");
	self.condition.fill_structure_string(buffer);
	buffer.push_str(format!(" {}", self.then_line).as_str());

    }
}

pub struct GwRem {
    pub comment : String
}

impl GwInstruction for GwRem {
    fn eval (&self, _line: i16, _context : &mut EvaluationContext) -> InstructionResult{
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"REM ");
        buffer.push_str(&self.comment[..]);
    }
}


pub struct GwCls {
    
}

impl GwInstruction for GwCls {
    fn eval (&self, _line: i16, _context : &mut EvaluationContext) -> InstructionResult{
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"CLS");
    }
}

pub struct GwAssign {
    pub variable : String,
    pub expression : Box<dyn GwExpression>
}

impl GwInstruction for GwAssign {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        let expression_evaluation = self.expression.eval(context);
        context.set_variable(&self.variable, &expression_evaluation);
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.variable[..]);
        buffer.push_str(&" = ");
        self.expression.fill_structure_string(buffer);
    }    
}

//

pub struct GwArrayAssign {
    pub variable : String,
    pub indices_expressions : Vec<Box<dyn GwExpression>>,
    pub expression : Box<dyn GwExpression>
}

impl GwInstruction for GwArrayAssign {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        let mut evaluated_arguments = Vec::with_capacity(self.indices_expressions.len());
        for arg in &self.indices_expressions {
            let eval_index = arg.eval(context);
            evaluated_arguments.push(try_to_get_integer_index(&eval_index).unwrap());
        }

        let expression_evaluation = self.expression.eval(context);
        context.set_array_entry(
            &self.variable,
            evaluated_arguments,
            &expression_evaluation);
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.variable[..]);
        buffer.push_str(&"(");
        for arg in &self.indices_expressions {
            arg.fill_structure_string(buffer);
        }

        buffer.push_str(&") = ");
        self.expression.fill_structure_string(buffer);
    }    
}


//


pub struct GwGotoStat {
    pub line : i16
}

impl GwInstruction for GwGotoStat {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{
        if let Some(actual_line) =  context.get_real_line(self.line) {
            return InstructionResult::EvaluateLine(actual_line);
        } else {
            println!("-- {}", self.line);
            panic!("Trying to jump to non existing line");
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("GOTO ");
    }
}

pub enum SwitchIndicator {
    On, Off
}

pub struct GwKeyStat {
    pub indicator : SwitchIndicator
}

impl GwInstruction for GwKeyStat {
    fn eval (&self, _line: i16, _context : &mut EvaluationContext) -> InstructionResult{
       InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"KEY ");
        match self.indicator {
            SwitchIndicator::On => buffer.push_str("ON"),
            SwitchIndicator::Off => buffer.push_str("OFF")
        }
    }
}

pub struct GwColor {
    pub red : Box<dyn GwExpression>,
    pub green : Box<dyn GwExpression>,
    pub blue : Box<dyn GwExpression>    
}

impl GwInstruction for GwColor {
    fn eval (&self, _line: i16, _context : &mut EvaluationContext) -> InstructionResult{
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"COLOR ");
        self.red.fill_structure_string(buffer);
        buffer.push_str(&", ");
        self.green.fill_structure_string(buffer);
        buffer.push_str(&", ");
        self.blue.fill_structure_string(buffer);
    }
}

pub enum PrintSeparator {
    Comma,
    Semicolon
}

pub struct GwPrintStat {
    pub expressions : Vec<(Option<Box<dyn GwExpression>>, Option<PrintSeparator>)>
}

impl GwInstruction for GwPrintStat {
    fn eval (&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult{

        let mut result = String::new();
        let mut i = 0;
        let mut newline_at_the_end = true;
        for print_expr in &self.expressions {
            match print_expr {
                (Some(expr), separator) => {
                    let evaluated_expr = expr.eval(context);
                    result.push_str(&evaluated_expr.to_string()[..]);
                    if i == &self.expressions.len() - 1 {
                        if let Some(PrintSeparator::Semicolon) = separator {
                            newline_at_the_end = false;
                        }
                    }
                },
                _ => {}
            }
            i += 1;
        }

        if newline_at_the_end {
            println!("{}", result.to_string());
        } else {
            print!("{}", result.to_string());
        }
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"PRINT ");

//        let mut result = String::new();
        for print_expr in &self.expressions {
            match print_expr {
                (Some(expr), _) => {
                    expr.fill_structure_string(buffer);
                },
                _ => {}
            }
            buffer.push_str(";");
        }
    }
}

pub struct GwInputStat {
    pub prompt : Option<String>,
    pub variables : Vec<Box<dyn GwAssignableExpression> >
}

fn read_variable_from_input(variable: &Box<dyn GwAssignableExpression>,
			    context: &mut EvaluationContext,
			    str_value: &str) {
    match variable.get_type(context) {
	GwVariableType::Double => {
	    variable.assign_value(
		ExpressionEvalResult::DoubleResult(str_value.trim_end().parse::<f32>().unwrap()),
		context);
        } 
	_ => panic!("Not implemented INPUT for this type")
    }
}

impl GwInstruction for GwInputStat {
    fn eval(&self, _line: i16, context : &mut EvaluationContext) -> InstructionResult {
        let mut buffer = String::new();
	let mut pr = "?";
	if let Some(ref prompt) = self.prompt {
	    pr = prompt.as_str();
	}
	print!("{}", pr);
	io::stdout().flush().expect("Success");
        io::stdin().read_line(&mut buffer).expect("Success");

	let mut var_idx = 0;
	for part in buffer.split(',') {
	    read_variable_from_input(&self.variables[var_idx], context, part);
	    var_idx = var_idx + 1;
	}

        // match context.get_variable_type(&self.variables) {
        //     Some(GwVariableType::Double) => {
        //         context.set_variable(
        //             &self.variable,
        //             &ExpressionEvalResult::DoubleResult(buffer.trim_end().parse::<f32>().unwrap()));
        //     },
        //     None => { 
        //         // Assume `double` for non-existing variable
        //         context.set_variable(
        //             &self.variable,
        //             &ExpressionEvalResult::DoubleResult(buffer.trim_end().parse::<f32>().unwrap()));

        //     },
        //     _ => panic!("Not implemented INPUT for this type")
        // }
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"INPUT ");
	match &(self.prompt)  {
	    Some(ptp) => {
		buffer.push_str(ptp.as_str());
		buffer.push_str(",");
	    }
	    _ => {}
	}
	let mut i = 0;
	for variable in &self.variables {
	    (*variable).fill_structure_string(buffer);
	    if i != self.variables.len() - 1 {
		buffer.push_str(",");
	    }
	    i = i + 1;
	}
    }
}




#[cfg(test)]
mod eval_tests {
    use std::collections::HashMap;
    
    // use crate::gwparser::GwAssign;
    // use crate::gwparser::GwIntegerLiteral;
    // use crate::gwparser::ProgramLine;
    // use crate::gwparser::GwProgram;
    // use crate::gwparser::EvaluationContext;
    // use crate::gwparser::ExpressionEvalResult;
    //    use crate::parser::*;
    use crate::eval::ExpressionEvalResult;
    use crate::eval::*;
    
    #[test]
    fn it_tests_basic_eval() {
        let line1 = ProgramLine {
            line: 10,
            instruction: Box::new(GwAssign {
                variable: String::from("X"),
                expression: Box::new( GwIntegerLiteral {
                    value: 10
                })
            }),
            rest_instructions: None
        };

        let program  = GwProgram {
            lines: vec![line1]
        };

        let mut context = EvaluationContext {
            variables: HashMap::new(),
            array_variables: HashMap::new(),
            jump_table: HashMap::new(),
            underlying_program: None,
	    pair_instruction_table: HashMap::new(),
	    real_lines: Some(vec![
		&program.lines.get(0).unwrap().instruction
	    ])
        };

        context.variables.insert(
	    String::from("X"),
	    ExpressionEvalResult::IntegerResult(5));

        if let Some(ExpressionEvalResult::IntegerResult(value)) = context.lookup_variable(&String::from("X")) {
            let some_value : i16 = 5;
            assert_eq!(&some_value, value);
        }

        program.eval(&mut context);

        if let Some(ExpressionEvalResult::IntegerResult(value)) = context.lookup_variable(&String::from("X")) {
            let some_value : i16 = 10;
            assert_eq!(&some_value, value);
        }        
    }

    #[test]
    fn it_negates_integer_expressions() {
        let negation = GwNegExpr {
            expr: Box::new(GwIntegerLiteral::with_value(1))
        };

        let mut context = empty_context();

        match negation.eval(&mut context) {
            ExpressionEvalResult::IntegerResult(x) => assert_eq!(x, -1),
            _ => assert!(false)
        }
    }

    #[test]
    fn it_negates_double_expressions() {
        let negation = GwNegExpr {
            expr: Box::new(GwDoubleLiteral::with_value(2.5))
        };

        let mut context = empty_context();

        match negation.eval(&mut context) {
            ExpressionEvalResult::DoubleResult(x) => assert_eq!(x, -2.5),
            _ => assert!(false)
        }
    }    

    #[test]
    fn it_tests_basic_array_eval() {
        let line1 = ProgramLine {
            line: 10,
            instruction: Box::new(GwArrayAssign {
                variable: String::from("A"),
                indices_expressions: vec![Box::new(GwIntegerLiteral::with_value(1))],
                expression: Box::new( GwIntegerLiteral {
                    value: 12
                })
            }),
            rest_instructions: None
        };

        let program  = GwProgram {
            lines: vec![line1]
        };

        let mut context = EvaluationContext::new();
	context.real_lines = Some(vec![]);

        context.declare_array(String::from("A"), 10);

        let arr1 = context.get_existing_array(&String::from("A"), 1);
        
        if let ExpressionEvalResult::IntegerResult(value) = arr1.unwrap().get_value(vec![1]) {
            let some_value : i16 = 0;
            assert_eq!(some_value, value);
        }

        program.eval(&mut context);

        let arr2 = context.get_existing_array(&String::from("A"), 1);

        if let ExpressionEvalResult::IntegerResult(value) = arr2.unwrap().get_value(vec![1]) {
            let some_value : i16 = 12;
            assert_eq!(some_value, value);
        }        
    }



    fn empty_context() -> EvaluationContext<'static> {
        EvaluationContext {
            variables: HashMap::new(),
            array_variables: HashMap::new(),
            jump_table: HashMap::new(),
            underlying_program: None,
	    pair_instruction_table: HashMap::new(),
	    real_lines: None
        }
    }

}

