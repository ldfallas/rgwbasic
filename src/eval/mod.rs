
pub mod binary;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::io::prelude::*;
use std::process::exit;
use crate::parser::ParserResult;
use crate::parser::parse_instruction_line_from_string;

enum GwVariableType {
    Double,
    Integer,
    String
}


pub enum InstructionResult {
    EvaluateNext,
    EvaluateLine(i16),
    EvaluateEnd
}

pub trait GwInstruction {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult;
    fn fill_structure_string(&self,   buffer : &mut String);
}

#[derive(Clone)]
pub enum ExpressionEvalResult {
    StringResult(String),
    IntegerResult(i16),
    DoubleResult(f32)
}

fn get_default_value_for_type(var_type : &GwVariableType) -> ExpressionEvalResult {
    match var_type {
        GwVariableType::String => ExpressionEvalResult::StringResult(String::from("")),
        GwVariableType::Integer => ExpressionEvalResult::IntegerResult(0),
        GwVariableType::Double => ExpressionEvalResult::DoubleResult(0.0)
    }       
}


impl ExpressionEvalResult {
    fn to_string(&self) -> String {
        match self {
            ExpressionEvalResult::StringResult(some_string) => some_string.clone(),
            ExpressionEvalResult::IntegerResult(iresult) => String::from(iresult.to_string()),
            ExpressionEvalResult::DoubleResult(dresult) => String::from(dresult.to_string())
            
        }
    }
}

pub struct EvaluationContext<'a> {
    variables: HashMap<String, ExpressionEvalResult>,
    array_variables: HashMap<String, GwArray>,
    jump_table: HashMap<i16, i16>,
    underlying_program: Option<&'a mut GwProgram>
}

impl EvaluationContext<'_>  {
    pub fn new() -> EvaluationContext<'static>   {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            array_variables: HashMap::new(),
            underlying_program: None
        }        
    }
    pub fn with_program(programm : &mut GwProgram) -> EvaluationContext {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            array_variables: HashMap::new(),
            underlying_program: Some(programm)
        }
    }

    pub fn set_array_entry(&mut self, name : &String, indices : Vec<u16>, new_value : &ExpressionEvalResult) {
        if let Some(mut_array) = self.array_variables.get_mut(name) {
            mut_array.set_value(&indices, &new_value);
        } else {
            panic!("array not found");
        }
    }

    pub fn declare_array(&mut self, name : String, size : u16) {
        let new_array = GwArray::new_one_dimension(size, GwVariableType::Double);
        self.array_variables.insert(name, new_array);
    }

    pub fn get_existing_array(&self, name : &String, _size : usize) -> Option<&GwArray> {
        self.array_variables.get(name)
    }

    fn get_existing_function(&self, _name : &String, _size : usize) -> Option<&GwArray> {
        panic!("not implemented");
    }


    fn get_real_line(&self, referenced_line : i16) -> Option<i16> {
        if let Some(lin) =  self.jump_table.get(&referenced_line) {
            Some(*lin)
        } else {
            None
        }
    }   
    
    fn lookup_variable(&self, name : &String) -> Option<&ExpressionEvalResult> {
        self.variables.get(name)
    }
    
    fn set_variable(&mut self, name : &String, value : &ExpressionEvalResult) {
        self.variables.insert(name.clone(), value.clone());
    }

    fn get_variable_type(&self, name : &String) -> Option<GwVariableType> {
        match self.lookup_variable(name) {
            Some(ExpressionEvalResult::StringResult(_)) => Some(GwVariableType::String),
            Some(ExpressionEvalResult::IntegerResult(_)) => Some(GwVariableType::Integer),
            Some(ExpressionEvalResult::DoubleResult(_)) => Some(GwVariableType::Double),
            _ => None
        }
    }
}

pub trait GwExpression {
    fn eval(&self, context : &mut EvaluationContext) -> ExpressionEvalResult ;
    fn fill_structure_string(&self,   buffer : &mut String);
}

pub struct GwArray {
    values : Vec<ExpressionEvalResult>,
    element_type : GwVariableType,
    dimensions: Vec<u16>
}

impl GwArray {
    fn new_one_dimension(size : u16, array_type : GwVariableType)
                         -> GwArray{
        let mut values = Vec::with_capacity(usize::from(size));
        for _i in 0..size{
            values.push( get_default_value_for_type(&array_type));
        }
        let dimensions = vec![size];
        GwArray {
            values: values,
            element_type: array_type,
            dimensions: dimensions
        }
    }

    pub fn get_value(&self, index_array : Vec<u16>) -> ExpressionEvalResult {
        let mut index : u16 = 0;
        for i in 1..self.dimensions.len() {
            index = (index_array[i] as u16) * self.dimensions[i];
        }
        index = index + (index_array[index_array.len() - 1] as u16);
        self.values[usize::from(index)].clone()
    }

    pub fn set_value(&mut self, index_array : &Vec<u16>, new_value : &ExpressionEvalResult) {
        let mut index : u16 = 0;
        for i in 1..self.dimensions.len() {
            index = (index_array[i] as u16) * self.dimensions[i];
        }
        index = index + (index_array[index_array.len() - 1] as u16);
        self.values[usize::from(index)] = new_value.clone();
    }
}

pub struct GwParenthesizedAccessExpr {
    name : String,
    arguments : Vec<Box<dyn GwExpression>>
}

impl GwParenthesizedAccessExpr {
    fn new(name : String, arguments : Vec<Box<dyn GwExpression>>)
           -> GwParenthesizedAccessExpr
    {
        GwParenthesizedAccessExpr {
            name: name,
            arguments: arguments
        }
    }    
}

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
        //self.expr.fill_structure_string(buffer);
        buffer.push_str(")");        
    }

}

pub struct GwCall {
    pub array_or_function : String,
    pub arguments : Vec<Box<dyn GwExpression>>
}

impl GwExpression for GwCall {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        panic!("Not implemented");
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.array_or_function[..]);
        buffer.push_str("(");
        for arg in &self.arguments {
            arg.fill_structure_string(buffer);
        }
        buffer.push_str(")");        
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
        GwStringLiteral { value: value } 
    }
    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&self.value[..]);
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
        GwIntegerLiteral { value : value }
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
        GwDoubleLiteral { value : value }
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

impl GwVariableExpression {
    pub fn with_name(name : String) -> GwVariableExpression {
        GwVariableExpression { name: name }
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




pub struct ProgramLine {
    pub line : i16,
    pub instruction : Box<dyn GwInstruction>,
    pub rest_instructions : Option<Vec<Box<dyn GwInstruction>>>
}

impl ProgramLine {
    fn get_line(&self) -> i16 {
        self.line
    }
    
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult {
         self.instruction.eval(context)
    }

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
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, _context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, _context : &mut EvaluationContext) -> InstructionResult{
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
            condition: condition,
            then_line: then_line
        };
    }
}

impl GwInstruction for GwIf {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        match self.condition.eval(context) {
            ExpressionEvalResult::IntegerResult(i_result) if i_result == 0 => InstructionResult::EvaluateNext,
            ExpressionEvalResult::DoubleResult(d_result) if d_result == 0.0 => InstructionResult::EvaluateNext,
            _ => InstructionResult::EvaluateLine(self.then_line)
        }        
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"IF ...");
    }
}

pub struct GwRem {
    pub comment : String
}

impl GwInstruction for GwRem {
    fn eval (&self, _context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, _context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, _context : &mut EvaluationContext) -> InstructionResult{
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
    fn eval (&self, _context : &mut EvaluationContext) -> InstructionResult{
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

pub struct GwPrintStat {
    pub expressions : Vec<Option<Box<dyn GwExpression>>>
}

impl GwInstruction for GwPrintStat {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{

        let mut result = String::new();
        for print_expr in &self.expressions {
            match print_expr {
                Some(expr) => {
                    let evaluated_expr = expr.eval(context);
                    result.push_str(&evaluated_expr.to_string()[..]);
                },
                _ => {}
            }
        }

        println!("{}", result.to_string());
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"PRINT ");

//        let mut result = String::new();
        for print_expr in &self.expressions {
            match print_expr {
                Some(expr) => {
                    expr.fill_structure_string(buffer);
                },
                _ => {}
            }
            buffer.push_str(";");
        }
    }
}

pub struct GwInputStat {
    pub variable : String
}

impl GwInstruction for GwInputStat {
    fn eval(&self, context : &mut EvaluationContext) -> InstructionResult {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer);
        match context.get_variable_type(&self.variable) {
            Some(GwVariableType::Double) => {
                context.set_variable(
                    &self.variable,
                    &ExpressionEvalResult::DoubleResult(buffer.trim_end().parse::<f32>().unwrap()));
            },
            None => {
                // Assume `double` for non-existing variable
                context.set_variable(
                    &self.variable,
                    &ExpressionEvalResult::DoubleResult(buffer.trim_end().parse::<f32>().unwrap()));

            },
            _ => panic!("Not implemented INPUT for this type")
        }
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"INPUT ");
        buffer.push_str(&self.variable[..]);
    }
}


pub struct GwProgram {
    lines : Vec<ProgramLine>,
//    lines_index : Vec<u16>
}

impl GwProgram {
    pub fn new() -> GwProgram {
        GwProgram {
            lines: Vec::new(),
        }
    }

    pub fn load_from(&mut self, file_name : &str) -> io::Result<()> {
        let f = File::open(file_name)?;
        let reader = BufReader::new(f);
        let mut line_number = 1;
        for line in reader.lines() {
            let uline = line.unwrap();
            println!(">> {}", uline);
            match parse_instruction_line_from_string(uline) {
                ParserResult::Success(parsed_line) => self.add_line(parsed_line),
                ParserResult::Error(error) => {println!("Line {} Error: {}", line_number, error); break;},
                ParserResult::Nothing=> println!("Nothing")       
            }
            line_number = line_number + 1;
        }
        Ok(())
    }

    pub fn list(&self) {
        for element in self.lines.iter() {
            let mut string_to_print = String::new();
            element.fill_structure_string(&mut string_to_print);
            println!("{}", string_to_print);
        }
    }

    pub fn run(&self) {
        let mut table = HashMap::new();
        let mut i = 0;
        for e in self.lines.iter() {
            table.insert(e.get_line(), i);
            i = i + 1;
        }
        let mut context = EvaluationContext {
            array_variables: HashMap::new(),
            variables: HashMap::new(),
            jump_table: table,
            underlying_program: None
        };
        self.eval(&mut context);
    }
    
    pub fn add_line(&mut self, new_line : ProgramLine) {
        let mut i = 0;        
        while i < self.lines.len() {
            if new_line.get_line() == self.lines[i].get_line() {
                self.lines[i] = new_line;
                return;
            } else if new_line.get_line() < self.lines[i].get_line() {
                self.lines.insert(i, new_line);
                return;
            }
            i = i + 1;
        }
        self.lines.push(new_line);
    }
    
    fn eval(&self, context : &mut EvaluationContext) {
        let mut current_index = 0;
        loop {
            if current_index >= self.lines.len() {
                break;
            }

            let eval_result = self.lines[current_index].eval(context);
            match eval_result {
                InstructionResult::EvaluateNext => {
                    current_index = current_index + 1;
                }
                InstructionResult::EvaluateLine(new_line) => {
                    current_index = usize::try_from(new_line).unwrap();
                }
                InstructionResult::EvaluateEnd => {
                    break;
                }
            }            
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
            underlying_program: None                
        };

        context.variables.insert(String::from("X"), ExpressionEvalResult::IntegerResult(5));

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

}

