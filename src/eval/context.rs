use std::collections::HashMap;
use std::convert::TryFrom;
use crate::parser::parse_instruction_line_from_string;
use crate::parser::ParserResult;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use std::io::BufReader;

pub enum GwVariableType {
    Double,
    Integer,
    String
}

pub struct ProgramLine {
    pub line : i16,
    pub instruction : Box<dyn GwInstruction>,
    pub rest_instructions : Option<Vec<Box<dyn GwInstruction>>>
}


pub enum InstructionResult {
    EvaluateNext,
    EvaluateLine(i16),
    EvaluateEnd,
    EvaluateToError(String)
}

pub trait GwInstruction {
    fn eval (&self, line: i16, context : &mut EvaluationContext) -> InstructionResult;
    fn fill_structure_string(&self,   buffer : &mut String);
    fn is_wend(&self) -> bool { false }
    fn is_while(&self) -> bool { false }     
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
    pub fn to_string(&self) -> String {
        match self {
            ExpressionEvalResult::StringResult(some_string) => some_string.clone(),
            ExpressionEvalResult::IntegerResult(iresult) => String::from(iresult.to_string()),
            ExpressionEvalResult::DoubleResult(dresult) => String::from(dresult.to_string())
            
        }
    }
}

pub struct EvaluationContext<'a> {
    pub variables: HashMap<String, ExpressionEvalResult>,
    pub array_variables: HashMap<String, GwArray>,
    pub jump_table: HashMap<i16, i16>,
    pub underlying_program: Option<&'a mut GwProgram>,   
    pub pair_instruction_table: HashMap<i16, i16>,
    pub real_lines: Option<Vec<& 'a Box<dyn GwInstruction>>>,
}

impl EvaluationContext<'_>  {
    pub fn new() -> EvaluationContext<'static>   {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            array_variables: HashMap::new(),
            underlying_program: None,
            pair_instruction_table: HashMap::new(),
            real_lines: None
        }        
    }
    pub fn with_program(program : &mut GwProgram) -> EvaluationContext {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            array_variables: HashMap::new(),
            underlying_program: Some(program),
            pair_instruction_table: HashMap::new(),
            real_lines: None
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

    pub fn get_existing_function(&self, _name : &String, _size : usize) -> Option<&GwArray> {
        panic!("not implemented");
    }


    pub fn get_real_line(&self, referenced_line : i16) -> Option<i16> {
        if let Some(lin) =  self.jump_table.get(&referenced_line) {
            Some(*lin)
        } else {
            None
        }
    }   
    
    pub fn lookup_variable(&self, name : &String) -> Option<&ExpressionEvalResult> {
        self.variables.get(name)
    }
    
    pub fn set_variable(&mut self, name : &String, value : &ExpressionEvalResult) {
        self.variables.insert(name.clone(), value.clone());
    }

    pub fn get_variable_type(&self, name : &String) -> Option<GwVariableType> {
        match self.lookup_variable(name) {
            Some(ExpressionEvalResult::StringResult(_)) => Some(GwVariableType::String),
            Some(ExpressionEvalResult::IntegerResult(_)) => Some(GwVariableType::Integer),
            Some(ExpressionEvalResult::DoubleResult(_)) => Some(GwVariableType::Double),
            _ => None
        }
    }
}


pub struct GwArray {
    values : Vec<ExpressionEvalResult>,
//    element_type : GwVariableType,
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
            values,
//            element_type: array_type,
            dimensions
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


pub struct GwProgram {
    pub lines : Vec<ProgramLine>,
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
            underlying_program: None,
            pair_instruction_table: HashMap::new(),
            real_lines: Some(vec![])
        };
        //      for j in 1..self.lines.len() {
        
        {
            let real_lines = &mut context.real_lines.as_mut().expect("the vec");
            for e in self.lines.iter() {
                real_lines.push(&e.instruction);
                if let Some(ref rest) = e.rest_instructions {
                    for nested in rest {
                        real_lines.push(&nested);
                    }
                }           
            }
        }
        
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
    
    pub fn eval(&self, context : &mut EvaluationContext) {
        let mut current_index = 0;
        loop {
            let real_lines = &context.real_lines.as_ref().expect("real_lines calculated");
            if current_index >= real_lines.len() {      
                break;
            }

            let eval_result = real_lines[current_index].eval(current_index as i16, context);
            match eval_result {
                InstructionResult::EvaluateNext => {
                    current_index = current_index + 1;
                }
                InstructionResult::EvaluateLine(new_line) => {
                    current_index = usize::try_from(new_line).unwrap();
                }
                InstructionResult::EvaluateEnd => {
                    break;
                },
                InstructionResult::EvaluateToError(error_message) => {
                    println!("RUNTIME ERROR: {}", error_message);
                    break;
                }
            }            
        }
    }
}
