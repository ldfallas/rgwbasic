
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::io::prelude::*;
use std::process::exit;
use crate::parser::ParserResult;
use crate::parser::parse_instruction_line_from_string;

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
    IntegerResult(i16)    
}

impl ExpressionEvalResult {
    fn to_string(&self) -> String {
        match self {
            ExpressionEvalResult::StringResult(some_string) => some_string.clone(),
            ExpressionEvalResult::IntegerResult(iresult) => String::from(iresult.to_string())
        }
    }
}

pub struct EvaluationContext<'a> {
    variables: HashMap<String, ExpressionEvalResult>,
    jump_table: HashMap<i16, i16>,
    underlying_program: Option<&'a mut GwProgram>
}

impl EvaluationContext<'_>  {
    pub fn with_program(programm : &mut GwProgram) -> EvaluationContext {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            underlying_program: Some(programm)
        }
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
}

pub trait GwExpression {
    fn eval(&self, context : &mut EvaluationContext) -> ExpressionEvalResult ;
    fn fill_structure_string(&self,   buffer : &mut String);
}

pub struct GwStringLiteral {
    value : String
}

impl GwStringLiteral {
    pub fn with_value(value : String) -> GwStringLiteral {
        GwStringLiteral { value: value } 
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


pub struct GwBinaryOperation {
    pub kind: GwBinaryOperationKind,
    pub left: Box<dyn GwExpression>,
    pub right: Box<dyn GwExpression>
}

impl GwBinaryOperation {
   fn fill_operator(&self, buffer : &mut String) {
        match self.kind {
            GwBinaryOperationKind::Plus => buffer.push_str(" + "),
            GwBinaryOperationKind::Times => buffer.push_str(" * "),
            GwBinaryOperationKind::Minus => buffer.push_str(" - "),
            
            _ => buffer.push_str(" ?? ")
        }
    }
}

impl GwExpression for GwBinaryOperation {
    fn eval (&self, context : &mut EvaluationContext) -> ExpressionEvalResult {
        let left_result = self.left.eval(context);
        let right_result = self.right.eval(context);
        match (left_result, right_result) {
            (ExpressionEvalResult::IntegerResult(left),
             ExpressionEvalResult::IntegerResult(right)) =>
                ExpressionEvalResult::IntegerResult(left + right),
            (_, _) => panic!("Not implemented")
            
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
    pub expression : Box<dyn GwExpression>
}

impl GwInstruction for GwPrintStat {
    fn eval (&self, context : &mut EvaluationContext) -> InstructionResult{
        let result = self.expression.eval(context);
        println!("{}", result.to_string());
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str(&"PRINT ");
        self.expression.fill_structure_string(buffer);
    }
}

pub struct GwInputStat {
    pub variable : String
}

impl GwInstruction for GwInputStat {
    fn eval(&self, _context : &mut EvaluationContext) ->
        InstructionResult {
            
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
}

