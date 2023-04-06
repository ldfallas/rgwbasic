use std::rc::Rc;
use std::collections::HashMap;
use std::convert::TryFrom;
use crate::parser::parse_instruction_line_from_string;
use crate::parser::ParserResult;
use super::GwExpression;


#[derive(Debug)]
pub enum LineExecutionArgument {
    Empty,
    NextIteration,
    SubReturn,
    SupplyPendingResult(String)
}

pub struct ProgramLine {
    pub line : i16,
    pub instruction : Rc<dyn GwInstruction>,
    pub rest_instructions : Option<Vec<Rc<dyn GwInstruction>>>
}

#[derive(Debug)]
pub enum AsyncAction {
    ReadLine
}

#[derive(Debug)]
pub enum InstructionResult {
    EvaluateNext,
    EvaluateLine(i16),
    EvaluateLineWithArg(i16, LineExecutionArgument),
    EvaluateEnd,
    EvaluateToError(String),
    RequestAsyncAction(AsyncAction)
}

#[derive(Debug)]
pub enum EvalFragmentAsyncResult {
    YieldToLine(usize, LineExecutionArgument),
    EvaluationEnd,
    ReadLine(usize)
}

pub trait GwInstruction {
    fn eval(&self,
            line: i16,
            argument: LineExecutionArgument,
            context: &mut EvaluationContext,
            program: &mut GwProgram) -> InstructionResult;
    fn fill_structure_string(&self, buffer : &mut String);
    fn is_wend(&self) -> bool { false }
    fn is_while(&self) -> bool { false }
    fn is_for(&self) -> bool { false }
    fn is_next(&self) -> bool { false }
    fn get_data(&self) -> Option<&Vec<String>> { None }
}


#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum ExpressionEvalResult {
    StringResult(String),
    IntegerResult(i16),
    SingleResult(f32),
    DoubleResult(f64)
}

pub enum ExpressionType {
    String, Integer, Single, Double
}


fn get_default_value_for_type(var_type : &ExpressionType) -> ExpressionEvalResult {
    match var_type {
        ExpressionType::String => ExpressionEvalResult::StringResult(String::from("")),
        ExpressionType::Integer => ExpressionEvalResult::IntegerResult(0),
        ExpressionType::Single => ExpressionEvalResult::SingleResult(0.0),
        ExpressionType::Double => ExpressionEvalResult::DoubleResult(0.0)
    }
}


impl ExpressionEvalResult {
    pub fn to_string(&self) -> String {
        match self {
            ExpressionEvalResult::StringResult(some_string) => some_string.clone(),
            ExpressionEvalResult::IntegerResult(iresult) => String::from(iresult.to_string()),
            ExpressionEvalResult::SingleResult(dresult) => String::from(dresult.to_string()),
            ExpressionEvalResult::DoubleResult(dresult) => String::from(dresult.to_string())

        }
    }
}


pub trait Console {
    fn print(&mut self, value: &str);
    fn print_line(&mut self, value: &str);
    fn read_line(&mut self, buffer: &mut String);
    fn clear_screen(&mut self);
    fn current_text_column(&self) -> usize;
    fn read_file_lines(&self, file_name: &str) -> Box<dyn Iterator<Item=String>>;
    fn adjust_to_position(&mut self, position: usize) {
        let num_spaces: usize;
        let cur_column = self.current_text_column();
        if cur_column <= position {
            num_spaces = position - cur_column;
        } else {
            num_spaces = position;
            self.print_line("")
        }
       for _ in 1..num_spaces {
            self.print(" ");
        }
    }
    fn flush(&self);
    fn exit_program(&self);
    fn clone(&self) -> Box<dyn Console>;
    fn requires_async_readline(&self) -> bool { true }
//    fn read_line_async<F>(&self, result_handler: &F) where F: Fn(&str) -> ;
    fn log(&self, msg: &str) {
        println!("{}",msg)
    }
}



pub struct EvaluationContext/*<'a>*/ {
    pub variables: HashMap<String, ExpressionEvalResult>,
    pub array_variables: HashMap<String, GwArray>,
    pub jump_table: HashMap<i16, i16>,
//    pub underlying_program: Option<&'a mut GwProgram>,
    pub pair_instruction_table: HashMap<i16, i16>,
//    pub real_lines: Option<Vec<& 'a Box<dyn GwInstruction>>>,
//    pub data: Vec<& 'a String>,
    pub console: Box<dyn Console>,
    pub data_position: i32,
    pub subroutine_stack: Vec<i16>,
    pub current_real_line: i32
}



impl EvaluationContext/*<'_>*/  {
    pub fn new(console: Box<dyn Console>) -> EvaluationContext/*<'static>*/   {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            array_variables: HashMap::new(),
            //underlying_program: None,
            pair_instruction_table: HashMap::new(),
            //real_lines: None,
            //data: vec![],
            console,//: Box::new( DefaultConsole::new()),
            data_position: -1,
            subroutine_stack: vec![],
            current_real_line: -1,
        }
    }
    pub fn with_program(program : &mut GwProgram, console: Box<dyn Console>) -> EvaluationContext {
        EvaluationContext {
            variables : HashMap::new(),
            jump_table : HashMap::new(),
            array_variables: HashMap::new(),
            //underlying_program: Some(program),
            pair_instruction_table: HashMap::new(),
            //real_lines: None,
            //data: vec![],
            console,//: Box::new(DefaultConsole::new()),
            data_position: -1,
            subroutine_stack: vec![],
            current_real_line: -1,
        }
    }

    pub fn push_return(&mut self, line: i16) {
        self.subroutine_stack.push(line);
    }

    pub fn pop_return(&mut self) -> Option<i16> {
        self.subroutine_stack.pop()
    }

    pub fn set_array_entry(&mut self,
                           name : &str,
                           indices : Vec<usize>,
                           new_value : &ExpressionEvalResult) {
        if let Some(mut_array) = self.array_variables.get_mut(name) {
            mut_array.set_value(&indices, &new_value);
        } else {
            panic!("array not found");
        }
    }

    pub fn declare_array(&mut self, name : &str, size : usize) {
        let new_array = GwArray::new_one_dimension(size, ExpressionType::Double);
        self.array_variables.insert(String::from(name), new_array);
    }

    pub fn get_existing_array(&self, name : &str) -> Option<&GwArray> {
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

    pub fn lookup_variable(&self, name : &str) -> Option<&ExpressionEvalResult> {
        self.variables.get(name)
    }

    pub fn set_variable(&mut self, name : &str, value : &ExpressionEvalResult) -> Result<(), & 'static str> {
        if let Some(entry) = self.variables.get_mut(name) {
            if matches_type(entry, &value) {
                *entry = value.clone();
                Ok(())
            } else if let Some(new_value) = coerce_value_type(&value, entry) {
                *entry = new_value;
                Ok(())
            } else {
                Err("Type mismatch")
            }
        } else {
            self.variables.insert(name.to_string(), value.clone());
            Ok(())
        }
    }

    pub fn get_variable_type(&self, name : &String) -> Option<ExpressionType> {
        match self.lookup_variable(name) {
            Some(ExpressionEvalResult::StringResult(_)) => Some(ExpressionType::String),
            Some(ExpressionEvalResult::IntegerResult(_)) => Some(ExpressionType::Integer),
            Some(ExpressionEvalResult::SingleResult(_)) => Some(ExpressionType::Single),
            Some(ExpressionEvalResult::DoubleResult(_)) => Some(ExpressionType::Double),
            _ => None
        }
    }

    pub fn get_next_data_item<'a>(&mut self, program: &'a GwProgram) -> Option<&'a String> {
        self.data_position += 1;

        if let Some(ref the_value) = program.data.get(self.data_position as usize) {
            Some(the_value)
        } else {
            None
        }
    }

    pub fn set_variable_type(&mut self, name: &str, var_type: &ExpressionType) {
        match var_type {
            ExpressionType::String => {
                self.variables.insert(
                    name.to_string(),
                    ExpressionEvalResult::StringResult("".to_string()));
            }
            ExpressionType::Integer => {
                self.variables.insert(
                    name.to_string(),
                    ExpressionEvalResult::IntegerResult(0));
            }
            ExpressionType::Single => {
                self.variables.insert(
                    name.to_string(),
                    ExpressionEvalResult::SingleResult(0 as f32));
            }
            ExpressionType::Double => {
                self.variables.insert(
                    name.to_string(),
                    ExpressionEvalResult::DoubleResult(0 as f64));
            }
        }
    }

}

fn matches_type(entry: &ExpressionEvalResult, value: &ExpressionEvalResult) -> bool {
    match (entry, value) {
        (ExpressionEvalResult::IntegerResult(_), ExpressionEvalResult::IntegerResult(_)) => true,
        (ExpressionEvalResult::StringResult(_), ExpressionEvalResult::StringResult(_)) => true,
        (ExpressionEvalResult::SingleResult(_), ExpressionEvalResult::SingleResult(_)) => true,
        (ExpressionEvalResult::DoubleResult(_), ExpressionEvalResult::DoubleResult(_)) => true,
        _ => false
    }
}

fn coerce_value_type(from: &ExpressionEvalResult, to: &ExpressionEvalResult)
                     ->  Option<ExpressionEvalResult> {
    match (from, to) {
        // From int
        (ExpressionEvalResult::IntegerResult(int_value),
         ExpressionEvalResult::DoubleResult(_)) =>
            Some(ExpressionEvalResult::DoubleResult(*int_value as f64)),
        (ExpressionEvalResult::IntegerResult(int_value),
         ExpressionEvalResult::SingleResult(_)) =>
            Some(ExpressionEvalResult::SingleResult(*int_value as f32)),
        // from double
        (ExpressionEvalResult::DoubleResult(dbl_value),
         ExpressionEvalResult::SingleResult(_)) =>
            Some(ExpressionEvalResult::SingleResult(*dbl_value as f32)),
        (ExpressionEvalResult::DoubleResult(dbl_value),
         ExpressionEvalResult::IntegerResult(_)) =>
            Some(ExpressionEvalResult::IntegerResult(*dbl_value as i16)),
        // from single
        (ExpressionEvalResult::SingleResult(sgn_value),
         ExpressionEvalResult::DoubleResult(_)) =>
            Some(ExpressionEvalResult::DoubleResult(*sgn_value as f64)),
        (ExpressionEvalResult::SingleResult(sng_value),
         ExpressionEvalResult::IntegerResult(_)) =>
            Some(ExpressionEvalResult::IntegerResult(*sng_value as i16)),
        _ => None
    }
}


pub fn evaluate_to_usize(expr: &Box<dyn GwExpression>,
                         context: &mut EvaluationContext) -> Result<usize, String> {
    match expr.eval(context) {
        Ok(ExpressionEvalResult::IntegerResult(ival)) if ival >= 0 => Ok(ival as usize),
        Ok(ExpressionEvalResult::SingleResult(sval)) if sval >= 0.0 => Ok(sval as usize),
        Ok(ExpressionEvalResult::DoubleResult(dval)) if dval >= 0.0 => Ok(dval as usize),
        Err(eval_error) => Err(eval_error),
        _ => Err("Type mismatch".to_string())
    }
}


pub struct GwArray {
    values : Vec<ExpressionEvalResult>,
//    element_type : GwVariableType,
    dimensions: Vec<usize>
}

impl GwArray {
    fn new_one_dimension(size : usize, array_type : ExpressionType)
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

    pub fn get_value(&self, index_array : Vec<usize>) -> ExpressionEvalResult {
        let mut index : usize = 0;
        for i in 1..self.dimensions.len() {
            index = (index_array[i] as usize) * self.dimensions[i];
        }
        let final_index = index_array[index_array.len() - 1];
        index = index + ((final_index - 1) as usize);
        self.values[usize::from(index)].clone()
    }

    pub fn set_value(&mut self, index_array : &Vec<usize>, new_value : &ExpressionEvalResult) {
        let mut index : usize = 0;
        for i in 1..self.dimensions.len() {
            index = (index_array[i] as usize) * self.dimensions[i];
        }

        let final_index = index_array[index_array.len() - 1];
        index = index + ((final_index - 1) as usize);
        self.values[usize::from(index)] = new_value.clone();
    }
}

pub struct StepExecutionInfo/*<'a>*/ {
    pub last_step_result: InstructionResult,
//    pub execution_context: EvaluationContext/*<'a>*/
}

pub struct GwProgram {
    pub lines : Vec<ProgramLine>,
    pub real_lines: Vec<Rc<dyn GwInstruction>>,
    pub data: Vec<String>
}

impl GwProgram {
    pub fn new() -> GwProgram {
        GwProgram {
            lines: Vec::new(),
            real_lines: Vec::new(),
            data: Vec::new()                
        }
    }

    pub fn load_from(&mut self, file_name : &str, console: &Box<dyn Console>) -> Result<(), & 'static str> {
        // let f = File::open(file_name)?;
        // let reader = BufReader::new(f);
        // let mut line_number = 1;
        // for line in reader.lines() {
        let mut line_number = 1;
        for uline in console.read_file_lines(file_name) {
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

    pub fn list(&self, console: &mut Box<dyn Console>) {
        for element in self.lines.iter() {
            let mut string_to_print = String::new();
            element.fill_structure_string(&mut string_to_print);
            console.print_line(string_to_print.as_str());
        }
    }



    pub fn prepare_context(&mut self, console: &Box<dyn Console>) -> EvaluationContext {
        let real_lines = &mut self.real_lines;// &mut vec![];
        let mut global_data = vec![];
        let mut table = HashMap::new();
        let mut i = 0;

        for e in self.lines.iter() {
            table.insert(e.get_line(), i);
            real_lines.push(e.instruction.clone());
            if let Some(data) = e.instruction.get_data() {
                for data_item in data {
                    global_data.push(data_item);
                }
            }
            i += 1;
            if let Some(ref rest) = e.rest_instructions {
                for nested in rest {
                    real_lines.push(nested.clone());
                    i += 1;
                }
            }
        }

        let new_console = (*console).clone();
        let  context = EvaluationContext {
            array_variables: HashMap::new(),
            variables: HashMap::new(),
            jump_table: table,
            //underlying_program: None,
            pair_instruction_table: HashMap::new(),
            //real_lines: Some(real_lines.to_vec()),
            console: new_console,
            //data: global_data,
            data_position: -1,
            subroutine_stack: vec![],
            current_real_line: -1
        };
//        self.real_lines = *real_lines;
        return context;
    }

    pub fn start_step_execution(&mut self, console: &Box<dyn Console>) -> (StepExecutionInfo, EvaluationContext) {
        let mut context = self.prepare_context(console);
        let real_lines = &self.real_lines;// &context.real_lines.as_ref().expect("real_lines calculated");
        context.current_real_line = 0;
        let line = &real_lines[0].clone();
        let eval_result =
            line.eval(
                0,
                LineExecutionArgument::Empty,
                &mut context,
                self);

        (StepExecutionInfo {
//            execution_context: context,
            last_step_result: eval_result
        }, context)
    }

    pub fn step(&mut self,
                step_execution: &StepExecutionInfo,
                execution_context: &mut EvaluationContext) -> StepExecutionInfo {
        let mut context = /*step_execution.*/execution_context;
        let real_lines = &self.real_lines; //&context.real_lines.as_ref().expect("real_lines calculated");
        let mut arg = &LineExecutionArgument::Empty;
        let mut finish_execution = false;
        match &step_execution.last_step_result {
            InstructionResult::EvaluateNext => {
                context.current_real_line += 1;
            }
            InstructionResult::EvaluateLine(new_line) => {
                context.current_real_line = usize::try_from(*new_line).unwrap() as i32;
            }
            InstructionResult::EvaluateLineWithArg(new_line, result_arg) => {
                arg = result_arg;
                context.current_real_line = usize::try_from(*new_line).unwrap() as i32;
            }
            InstructionResult::EvaluateEnd => {
                finish_execution = true;
            },
            InstructionResult::EvaluateToError(error_message) => {
                context.console.print("RUNTIME ERROR: ");
                context.console.print_line(error_message.as_str());
                finish_execution = true;
            },
            InstructionResult::RequestAsyncAction(AsyncAction::ReadLine) => {
                context.console.print("Read line");
            }
            
        }

        let line = &real_lines[context.current_real_line as usize].clone();
        let new_eval_result =
            line.eval(
                0,
                LineExecutionArgument::Empty,
                &mut context,
                self);

        StepExecutionInfo {
            //execution_context: context,
            last_step_result: new_eval_result
        }
        
    }

    pub fn run(&mut self, console: &Box<dyn Console>) {
        let mut context = self.prepare_context(console);
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

    pub fn eval_fragment_async(&mut self,
                               line: usize,
                               line_execution_arg: LineExecutionArgument,
                               context: &mut EvaluationContext)
                            -> EvalFragmentAsyncResult{
//        let mut current_index = 0;
        //        let mut arg = LineExecutionArgument::Empty;
        let mut current_index = line;
        let mut arg = line_execution_arg;
        loop {
     //       let real_lines = &self.real_lines;//&context.real_lines.as_ref().expect("real_lines calculated");
            if current_index >= self.real_lines.len() {
                //break;
                return EvalFragmentAsyncResult::EvaluationEnd;
            }

            context.current_real_line = current_index as i32;
            let line = &self.real_lines[current_index].clone();
            let eval_result =
                     line.eval(
                         current_index as i16,
                         arg,
                         context,
                         self);
            arg = LineExecutionArgument::Empty;
            match eval_result {
                InstructionResult::EvaluateNext => {
                    current_index = current_index + 1;
                }
                InstructionResult::EvaluateLine(new_line) => {
                    current_index = usize::try_from(new_line).unwrap();
                }
                InstructionResult::EvaluateLineWithArg(new_line, result_arg) => {
                    arg = result_arg;
                    current_index = usize::try_from(new_line).unwrap();
                }
                InstructionResult::EvaluateEnd => {
                    //break;
                    return EvalFragmentAsyncResult::EvaluationEnd;
                    
                },
                InstructionResult::EvaluateToError(error_message) => {
                    context.console.print("RUNTIME ERROR: ");
                    context.console.print_line(error_message.as_str());
                    //break;
                    return EvalFragmentAsyncResult::EvaluationEnd;
                },
                InstructionResult::RequestAsyncAction(_) => {
                    //panic!("Attempting to execute async operation in sync evaluation");
                    return EvalFragmentAsyncResult::ReadLine(current_index);
                }                
            }
        }
    }

    pub fn eval(&mut self, context: &mut EvaluationContext) {
        let mut current_index = 0;
        let mut arg = LineExecutionArgument::Empty;
        loop {
     //       let real_lines = &self.real_lines;//&context.real_lines.as_ref().expect("real_lines calculated");
            if current_index >= self.real_lines.len() {
                break;
            }

            let line = &self.real_lines[current_index].clone();
            let eval_result =
                     line.eval(
                         current_index as i16,
                         arg,
                         context,
                         self);
            arg = LineExecutionArgument::Empty;
            match eval_result {
                InstructionResult::EvaluateNext => {
                    current_index = current_index + 1;
                }
                InstructionResult::EvaluateLine(new_line) => {
                    current_index = usize::try_from(new_line).unwrap();
                }
                InstructionResult::EvaluateLineWithArg(new_line, result_arg) => {
                    arg = result_arg;
                    current_index = usize::try_from(new_line).unwrap();
                }
                InstructionResult::EvaluateEnd => {
                    break;
                },
                InstructionResult::EvaluateToError(error_message) => {
                    context.console.print("RUNTIME ERROR: ");
                    context.console.print_line(error_message.as_str());
                    break;
                },
                InstructionResult::RequestAsyncAction(_) => {
                    panic!("Attempting to execute async operation in sync evaluation");
                }                
            }
        }
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use crate::eval::eval_tests::empty_context;

    #[test]
    fn it_declares_valid_array() -> Result<(), & 'static str> {
        let mut ctx = empty_context();
        ctx.declare_array("my_array", 3);

        ctx.set_array_entry(
            "my_array",
            vec![1],
            &ExpressionEvalResult::IntegerResult(10));
        ctx.set_array_entry(
            "my_array",
            vec![2],
            &ExpressionEvalResult::IntegerResult(20));
        ctx.set_array_entry(
            "my_array",
            vec![3],
            &ExpressionEvalResult::IntegerResult(30));

        if let Some(array) = ctx.get_existing_array("my_array") {
            let existing_values =
                (array.get_value(vec![1]),
                 array.get_value(vec![2]),
                 array.get_value(vec![3]));
            match existing_values {
                (ExpressionEvalResult::IntegerResult(10),
                 ExpressionEvalResult::IntegerResult(20),
                 ExpressionEvalResult::IntegerResult(30))
                    => Ok(()),
                _ => Err("Values not retrieved")
            }
        } else {
            Err("array not found")
        }
    }
}
