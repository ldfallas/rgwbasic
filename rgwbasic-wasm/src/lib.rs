mod utils;
use rgwbasic::eval::LineExecutionArgument;
use rgwbasic::{parser, eval};
use rgwbasic::eval::context::{AsyncAction,
                              Console,
                              StepExecutionInfo,
                              EvaluationContext,
                              EvalFragmentAsyncResult,
                              InstructionResult };
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{Request, RequestInit, RequestMode, Response};
use js_sys::{JsString, Promise};

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
    fn readLine(continueFunc: &Closure<dyn FnMut(String)>);
}

fn browser_request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("Cannot access 'window'")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Call to 'request_animation_frame'");
}

fn fetch_util_get(url: &str) ->  Promise {
    let mut options = RequestInit::new();
    options.method("GET");
    let request = Request::new_with_str_and_init(url, &options).expect("Request");
    request.headers().set("Accept", "text/plain");
    let window =
        web_sys::window().expect("Cannot access 'window'");
    window.fetch_with_request(&request)
}

#[wasm_bindgen]
pub struct GwInterpreterWrapper {
    interpreter: Rc<RefCell<GwWsmInterpreter>>
}

#[wasm_bindgen]
impl GwInterpreterWrapper {
    pub fn new() -> GwInterpreterWrapper {
        GwInterpreterWrapper {
            interpreter: Rc::new(RefCell::new(GwWsmInterpreter::new()))
        }
    }

    pub fn eval_in_interpreter(&mut self, command: &str) {
                utils::set_panic_hook();
        log("using wrapped(about)");
        let result = self.interpreter.borrow_mut().eval_in_interpreter(command, None);
        let interpreter1 = self.interpreter.clone();
        let cmd1 = String::from(command);
        if let Some(AsyncAction::LoadProgram(program_name)) = result {
            log("load async 2");
            let interpreter2 = interpreter1.clone();
            let closure = Closure::new(move |response:JsValue|{
                let cmd2 = cmd1.clone();
                log("load async 3");
                let interpreter3 = interpreter2.clone();
                let resp: Response = response.dyn_into().unwrap();
                let closure2 = Closure::new(move |response2:JsValue|{
                    let cmd3 = cmd2.clone();
                    log("load async 4");
                    let resp: JsString = response2.dyn_into().unwrap();
                    let code = resp.as_string().unwrap();//.split("\n");
                    interpreter3.borrow_mut().eval_in_interpreter(
                        &cmd3,
                        Some(LineExecutionArgument::SupplyPendingResult(code))
                    );
                    
                });
                resp.text().expect("No text").then(&closure2);
                closure2.forget();
            });
            fetch_util_get(&program_name).then(&closure);
            closure.forget();
        }
    }

    pub fn run_evaluator_loop(&mut self) {
        let mut interpreter = self.interpreter.clone();
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        log("Starting step execution wrapped");
        //interpreter.borrow_mut().start_step_execution();
        {
            interpreter.borrow_mut().create_execution_context();
        }
        
        *g.borrow_mut() = Some(Closure::new(move || {
            log("C");
            let _ = f.take();
            log("D");
            //log("stepping execution wrapped");
            //interpreter.borrow_mut().step_current_program();
            let result = interpreter.borrow_mut().run_async_fragment();
            GwInterpreterWrapper::evaluate_async_fragmen_result(
                result,
                interpreter.clone());
            // match result {
            //     EvalFragmentAsyncResult::EvaluationEnd => {
            //         log("Ending program execution");
            //         let _ = f.take();
            //     }
            //     EvalFragmentAsyncResult::ReadLine(line_to_continue) => {
            //         let f2 = Rc::new(RefCell::new(None));
            //         let g2 = f2.clone();                    
            //         let nf = f.clone();
            //         let interpreter_new = interpreter.clone();
            //         *g2.borrow_mut() = Some(Closure::new(move |result: String| {
            //             //alert(result.as_str());
            //             let _ = f2.take();
            //             interpreter_new.borrow_mut().continue_async_fragment(
            //                 line_to_continue,
            //                 LineExecutionArgument::SupplyPendingResult(result));
            //             //browser_request_animation_frame(nf.borrow().as_ref().unwrap());
                        
            //         }));
            //         readLine(g2.borrow().as_ref().unwrap());
            //     }
            //     the_result => {
            //         log(format!("ABOUT TO PANIC! Unimplemented async result: {:?}", the_result).as_str());
            //         todo!("Not implemented async result");
            //         browser_request_animation_frame(f.borrow().as_ref().unwrap());
            //     }
            // }
            
        }));
        log("A");
            browser_request_animation_frame(g.borrow().as_ref().unwrap());
            
        log("B");
    }

    fn evaluate_async_fragmen_result(
        result: EvalFragmentAsyncResult,
        interpreter: Rc<RefCell<GwWsmInterpreter>>) {
        match result {
            EvalFragmentAsyncResult::EvaluationEnd => {
                log("Ending program execution");
//                let _ = f.take();
            }
            EvalFragmentAsyncResult::YieldToLine(line_to_continue, execution_arg) => {
                let hold_closure_rc = Rc::new(RefCell::new(None));
                let actual_rc = hold_closure_rc.clone();
                let interpreter_new = interpreter.clone();
                *actual_rc.borrow_mut() = Some(Closure::new(move || {
                    log(format!("Ready to continue to: {},{:?}", line_to_continue, &execution_arg).as_str());
                    let new_result = interpreter_new.borrow_mut().continue_async_fragment(
                        line_to_continue,
                        execution_arg.clone()
                       // LineExecutionArgument::Empty
                    );
                    let _ = hold_closure_rc.take();
                    GwInterpreterWrapper::evaluate_async_fragmen_result(
                        new_result,
                        interpreter_new.clone());
                }));
                browser_request_animation_frame(actual_rc.borrow().as_ref().unwrap());
            }
            EvalFragmentAsyncResult::ReadLine(line_to_continue) => {
                let f2 = Rc::new(RefCell::new(None));
                let g2 = f2.clone();                    
//                let nf = f.clone();
                let interpreter_new = interpreter.clone();
                *g2.borrow_mut() = Some(Closure::new(move |result: String| {
                    //alert(result.as_str());
                    let _ = f2.take();
                    let new_result = interpreter_new.borrow_mut().continue_async_fragment(
                        line_to_continue,
                        LineExecutionArgument::SupplyPendingResult(result));
                    //browser_request_animation_frame(nf.borrow().as_ref().unwrap());
                    GwInterpreterWrapper::evaluate_async_fragmen_result(new_result, interpreter_new.clone());
                }));
                readLine(g2.borrow().as_ref().unwrap());
            }
            the_result => {
                log(format!("ABOUT TO PANIC! Unimplemented async result: {:?}", the_result).as_str());
                todo!("Not implemented async result");
                //browser_request_animation_frame(f.borrow().as_ref().unwrap());
            }
        }
    }
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

    fn requires_async_readline(&self) -> bool { true  }

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
pub struct GwWsmInterpreter {
    program: eval::GwProgram,
    console: Box<dyn Console>,
    current_execution_context: Option<EvaluationContext>,
    current_step:  Option<WsStepExecutionInfo>
}

//#[wasm_bindgen]
impl GwWsmInterpreter {
    pub fn new() -> GwWsmInterpreter {
        log("1>start");
        GwWsmInterpreter {
            program: eval::GwProgram::new(),
            console: Box::new(HtmlDivConsole {}),
            current_execution_context: None,
            current_step: None
//            last_step_info: None
        }
    }

    pub fn start_step_execution(&mut self) {
        self.current_step = Some(self.run_program_async());
    }

    pub fn step_current_program(&mut self) {
        // Notice here the use of `take`
        let mut moved = self.current_step.take().unwrap();
        self.current_step = Some(self.step_program(&mut moved));
    }

    fn run_program_async(&mut self) -> WsStepExecutionInfo {
        let console: Box<dyn Console> = Box::new(HtmlDivConsole{});
        let (first_result, ctx) =
            self.program.start_step_execution(&console);
        self.current_execution_context = Some(ctx);
        WsStepExecutionInfo::new(first_result, false)
    }

    fn create_execution_context(&mut self) {
        let console: Box<dyn Console> = Box::new(HtmlDivConsole{});

        self.current_execution_context = Some(self.program.prepare_context(&console));
        self.current_execution_context.as_mut().unwrap().current_real_line = 0;
    }

    pub fn run_async_fragment(&mut self) -> EvalFragmentAsyncResult {
        // self.program.eval_fragment_async(
        //     (&self.current_execution_context).as_ref().unwrap().current_real_line as usize,//0,
        //     LineExecutionArgument::Empty,
        //     self.current_execution_context.as_mut().unwrap())

        self.continue_async_fragment(
                (&self.current_execution_context).as_ref().unwrap().current_real_line as usize,
                LineExecutionArgument::Empty)
                    
    }

    pub fn continue_async_fragment(&mut self,
                                   line: usize,
                                   arg: LineExecutionArgument )
                                   -> EvalFragmentAsyncResult {
        self.program.eval_fragment_async(
            line,
            arg,
            self.current_execution_context.as_mut().unwrap())
    }

    fn step_program(&mut self, last_step: &mut WsStepExecutionInfo)
                        -> WsStepExecutionInfo {
        let mut mut_context = self.current_execution_context.as_mut().unwrap();
        let tmp_result =
            self.program.step(last_step.get_inner_execution(), mut_context);
        WsStepExecutionInfo::new(tmp_result, false)
    }
    
    pub fn eval_in_interpreter(&mut self, command: &str, evaluation_arg: Option<LineExecutionArgument>) -> Option<AsyncAction> {
//        let mut program = eval::GwProgram::new();
//        let mut console = HtmlDivConsole{};
//        console.print_line("Ok");
        //let mut uline = "PRINT (10 + 20)".to_string();
        let eval_arg = evaluation_arg.unwrap_or(eval::LineExecutionArgument::Empty);
        log("1. eval in interpreter");
        let mut uline = command.to_string();
        if !uline.is_empty() && uline.chars().next().unwrap().is_ascii_digit() {
            match parser::parse_instruction_line_from_string(uline) {
                parser::ParserResult::Success(parsed_line) => { self.program.add_line(parsed_line); }
                parser::ParserResult::Error(error) => { println!("Error: {}", error); }
                parser::ParserResult::Nothing => { println!("Nothing"); }
            };
        } else {
            log("2. ???");
            match parser::parse_repl_instruction_string (uline) {
                parser::ParserResult::Success(parsed_instr) => {
                    log("about to eval instruction");
                    let mut context = eval::EvaluationContext::with_program(&mut self.program, Box::new(HtmlDivConsole{}));
                    log("2. eval in interpreter__");
                    let result = parsed_instr.eval(-1,
                                      eval_arg,
                                      &mut context,
                                                   &mut self.program);
                    log("load async 1");
                    if let InstructionResult::RequestAsyncAction( 
                        async_action) = result {
                        return Some(async_action);
                    }
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
        None
    }
}
