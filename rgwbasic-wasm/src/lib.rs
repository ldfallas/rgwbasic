mod utils;
use rgwbasic::{parser, eval};
use rgwbasic::eval::context::{Console, StepExecutionInfo, EvaluationContext};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn log(s: &str);
    fn alert(s: &str);
    fn appendElementLd(s: &str);
    fn appendElementLn(s: &str);
}

struct HtmlDivConsole {
}

impl Console for HtmlDivConsole {
    fn print(&mut self, value: &str) {
        appendElementLd(value);
    }

    fn print_line(&mut self, value: &str) {        
        appendElementLn(value);
    }

    fn read_line(&mut self, buffer: &mut String) {
        //todo!()
        alert("read not yet implemented");
    }

    fn clear_screen(&mut self) {
        todo!()
    }

    fn current_text_column(&self) -> usize {
        todo!()
    }

    fn read_file_lines(&self, file_name: &str) -> Box<dyn Iterator<Item=String>> {
        todo!()
    }

    fn flush(&self) {

    }

    fn exit_program(&self) {
        todo!()
    }

    fn clone(&self) -> Box<dyn Console> {

        Box::new(HtmlDivConsole{})

    }
    fn log(&self, msg: &str) {
        log(msg);
    }
}


#[wasm_bindgen]
struct WsStepExecutionInfo {
    step_execution: StepExecutionInfo,
    pub finish: bool
}

impl WsStepExecutionInfo {
    fn new(step_execution: StepExecutionInfo, finish: bool) -> WsStepExecutionInfo {
        WsStepExecutionInfo {
            step_execution, finish
        }
    }

    fn get_inner_execution(&self) -> &StepExecutionInfo {
        &self.step_execution
    }
}

#[wasm_bindgen]
struct GwWsmInterpreter {
    program: eval::GwProgram,
    console: Box<dyn Console>,
    current_execution_context: Option<EvaluationContext>
}

#[wasm_bindgen]
impl GwWsmInterpreter {
    pub fn new() -> GwWsmInterpreter {
        log("1>start");
        GwWsmInterpreter {
            program: eval::GwProgram::new(),
            console: Box::new(HtmlDivConsole {}),
            current_execution_context: None
//            last_step_info: None
        }
    }

    pub fn run_program_async(&mut self) -> WsStepExecutionInfo {
        let console: Box<dyn Console> = Box::new(HtmlDivConsole{});
        let (first_result, ctx) =
            self.program.start_step_execution(&console);
        self.current_execution_context = Some(ctx);
        WsStepExecutionInfo::new(first_result, false)
    }

    pub fn step_program(&mut self, last_step: &mut WsStepExecutionInfo)
                        -> WsStepExecutionInfo {
        let mut mut_context = self.current_execution_context.as_mut().unwrap();
        let tmp_result =
            self.program.step(last_step.get_inner_execution(), mut_context);
        WsStepExecutionInfo::new(tmp_result, false)
    }
    
    pub fn eval_in_interpreter(&mut self, command: &str) {
//        let mut program = eval::GwProgram::new();
//        let mut console = HtmlDivConsole{};
//        console.print_line("Ok");
        //let mut uline = "PRINT (10 + 20)".to_string();
        log("1. eval in interpreter");
        let mut uline = command.to_string();

        if !uline.is_empty() && uline.chars().next().unwrap().is_ascii_digit() {
            match parser::parse_instruction_line_from_string(uline) {
                parser::ParserResult::Success(parsed_line) =>  self.program.add_line(parsed_line),
                parser::ParserResult::Error(error) => println!("Error: {}", error),
                parser::ParserResult::Nothing=> println!("Nothing")       
            }
        } else {            
            match parser::parse_repl_instruction_string (uline) {
                parser::ParserResult::Success(parsed_instr) => {
                    let mut context = eval::EvaluationContext::with_program(&mut self.program, Box::new(HtmlDivConsole{}));
                    log("2. eval in interpreter");
                    parsed_instr.eval(-1,
                                      eval::LineExecutionArgument::Empty,
                                      &mut context,
                                      &mut self.program);
                    context.console.flush();
                }
                parser::ParserResult::Error(msg) => {
                    self.console.print_line("Error parsing command");
                }
                _ => {
                    self.console.print_line("Error processing command");
                }
            }
        }
    }
}
