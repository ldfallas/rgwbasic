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
use js_sys::{JsString, Promise, Function};

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
    fn clearconsole();
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

    pub fn start_step(&self) -> Promise {

        Promise::new(
            &mut move |resolve, _reject| {
                let func_rc = Rc::new(resolve);
                self.interpreter.borrow_mut().start_step_program(func_rc);
            })
    } 

    pub fn step(&self) -> Promise  {
        Promise::new(
            &mut move |resolve, _reject| {
                let func_rc = Rc::new(resolve);
                self.interpreter.borrow_mut().evaluate_step_result(func_rc);
            })
    }
    
    pub fn real_vs_source_lines(&self, f: &Function) {
        self.interpreter.borrow_mut().real_vs_source_lines(f);
    }

    pub fn load_from_string(&mut self, code: &str) {
        let file_lines = Box::new(code.split("\n").map(|s| s.to_string()));
        let console = Box::new(HtmlDivConsole::new());
        //self.intepreter.load_from(&mut console, file_lines);
        self.interpreter.borrow_mut().load_from_string(code);
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

    pub fn run_evaluator_loop(&mut self) -> Promise {
       // let f = Rc::new(RefCell::new(None));
       // let g = f.clone();
       // *g.borrow_mut() = Some(Closure::new(move |resolve, reject| {
        // }));
        log("returning");

        Promise::new(
            &mut move |resolve, _reject| {
                let func_rc = Rc::new(resolve);
                self.run_evaluator_loop_internal(func_rc);
            })
    }

    fn run_evaluator_loop_internal(&mut self, resolve: Rc<Function>) {
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
                interpreter.clone(),
                resolve.clone());
        }));
        log("A");
            browser_request_animation_frame(g.borrow().as_ref().unwrap());
            
        log("B");
    }

    fn evaluate_async_fragmen_result(
        result: EvalFragmentAsyncResult,
        interpreter: Rc<RefCell<GwWsmInterpreter>>,
        resolve: Rc<Function>) {
        match result {
            EvalFragmentAsyncResult::EvaluationEnd => {
                log("Ending program execution");
                let _  = resolve.call0(&JsValue::NULL);
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
                        interpreter_new.clone(), resolve.clone());
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
                    GwInterpreterWrapper::evaluate_async_fragmen_result(new_result, interpreter_new.clone(), resolve.clone());
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
    column_position: usize
}

impl HtmlDivConsole {
    fn new() -> HtmlDivConsole {
        HtmlDivConsole {
            column_position: 0
        }
    }
}

impl Console for HtmlDivConsole {
    fn print(&mut self, value: &str) {
        appendElementLd(value);
        self.column_position += value.len();
    }

    
    fn print_line(&mut self, value: &str) {        
        appendElementLn(value);
        self.column_position = 0;
    }

    fn read_line(&mut self, buffer: &mut String) {
        //todo!()
        alert("read not yet implemented");
    }

    fn clear_screen(&mut self) {
        clearconsole();
    }

    fn current_text_column(&self) -> usize {
       self.column_position
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

        Box::new(HtmlDivConsole::new())

    }
    fn log(&self, msg: &str) {
        log(msg);
    }
}


#[wasm_bindgen]
pub struct JsCompatibleEvalFragmentAsyncResult {
    result: EvalFragmentAsyncResult
}

#[wasm_bindgen]
impl JsCompatibleEvalFragmentAsyncResult {
    fn new(inner_result: EvalFragmentAsyncResult) -> JsCompatibleEvalFragmentAsyncResult {
        JsCompatibleEvalFragmentAsyncResult {
            result: inner_result
        }
    }

    pub fn next_line(&self) -> usize {
        match self.result {
            EvalFragmentAsyncResult::YieldToLine(next_line, _) => next_line,
            _ => 0
        }
    }
}

// #[wasm_bindgen]
// struct WsStepExecutionInfo {
//     step_execution: StepExecutionInfo,
//     pub finish: bool
// }

// impl WsStepExecutionInfo {
//     fn new(step_execution: StepExecutionInfo, finish: bool) -> WsStepExecutionInfo {
//         WsStepExecutionInfo {
//             step_execution, finish
//         }
//     }

//     fn get_inner_execution(&self) -> &StepExecutionInfo {
//         &self.step_execution
//     }
// }

#[wasm_bindgen]
pub struct GwWsmInterpreter {
    program: eval::GwProgram,
    console: Box<dyn Console>,
    current_execution_context: Option<EvaluationContext>,
    //    current_step:  Option<WsStepExecutionInfo>
    current_step:  Option<EvalFragmentAsyncResult>
}

//#[wasm_bindgen]
impl GwWsmInterpreter {
    pub fn new() -> GwWsmInterpreter {
        log("1>start");
        GwWsmInterpreter {
            program: eval::GwProgram::new(),
            console: Box::new(HtmlDivConsole::new()),
            current_execution_context: None,
            current_step: None
//            last_step_info: None
        }
    }

    pub fn real_vs_source_lines(&self, f: &Function) {

        self.program.real_vs_source_lines(
            &mut |real, source| {
                let real_js = JsValue::from(real);
                let source_js = JsValue::from(source);
                f.call2(&JsValue::NULL, &real_js, &source_js);
            });
    }

    pub fn load_from_string(&mut self, code: &str) {
        let mut console: Box<dyn Console> = Box::new(HtmlDivConsole::new());
        let lines: Vec<String> = code.split("\n").map(|s| s.to_string()).collect();
        let file_lines = Box::new(lines.into_iter());
        self.program.load_from(&mut console, file_lines);
    }

    // pub fn start_step_execution(&mut self) {
    //     self.current_step = Some(self.run_program_async());
    // }

    // pub fn step_current_program(&mut self) {
    //     // Notice here the use of `take`
    //     let mut moved = self.current_step.take().unwrap();
    //     self.current_step = Some(self.step_program(&mut moved));
    // }

    // fn run_program_async(&mut self) -> WsStepExecutionInfo {
    //     let console: Box<dyn Console> = Box::new(HtmlDivConsole::new());
    //     let (first_result, ctx) =
    //         self.program.start_step_execution(&console);
    //     self.current_execution_context = Some(ctx);
    //     WsStepExecutionInfo::new(first_result, false)
    // }

    fn create_execution_context(&mut self) {
        let console: Box<dyn Console> = Box::new(HtmlDivConsole::new());

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
            None,
            self.current_execution_context.as_mut().unwrap())
    }


    pub fn start_step_program(&mut self, resolve: Rc<Function>) {
        self.create_execution_context();
        self.current_step = Some(EvalFragmentAsyncResult::YieldToLine(0, LineExecutionArgument::Empty));
//        self.step_program_async();
        self.evaluate_step_result(resolve);
    }
    // pub fn step_program_async(&mut self)  {
    //     let result = self.program.eval_fragment_async(
    //         line,
    //         arg,
    //         Some(1),
    //         self.current_execution_context.as_mut().unwrap());
    //     self.current_step = Some(result.clone());
    //     JsCompatibleEvalFragmentAsyncResult::new(result)
    // }

    fn evaluate_step_result(
        &mut self,
        resolve: Rc<Function>) {
        match &self.current_step {
            Some(EvalFragmentAsyncResult::EvaluationEnd) => {
                log("Ending program execution");
                let _  = resolve.call0(&JsValue::NULL);
//                let _ = f.take();
            }
            Some(EvalFragmentAsyncResult::YieldToLine(line_to_continue, execution_arg)) => {
                let result = self.program.eval_fragment_async(
                    *line_to_continue,
                    execution_arg.clone(),
                    Some(1),
                    self.current_execution_context.as_mut().unwrap());
                self.current_step = Some(result.clone());
                let mut resolved = JsValue::NULL;
                if let EvalFragmentAsyncResult::YieldToLine(line_to_continue, _) = result {
                    resolved = JsValue::from_f64(line_to_continue as f64);
                }
                let _  = resolve.call1(&JsValue::NULL, &resolved);
                
            }
            Some(EvalFragmentAsyncResult::ReadLine(line_to_continue)) => {
                // let f2 = Rc::new(RefCell::new(None));
                // let g2 = f2.clone();                    
                // let interpreter_new = interpreter.clone();
                // *g2.borrow_mut() = Some(Closure::new(move |result: String| {
                //     //alert(result.as_str());
                //     let _ = f2.take();
                //     let new_result = interpreter_new.borrow_mut().continue_async_fragment(
                //         line_to_continue,
                //         LineExecutionArgument::SupplyPendingResult(result));
                //     //browser_request_animation_frame(nf.borrow().as_ref().unwrap());
                //     GwInterpreterWrapper::evaluate_async_fragmen_result(new_result, interpreter_new.clone(), resolve.clone());
                // }));
                // readLine(g2.borrow().as_ref().unwrap());
                let _  = resolve.call0(&JsValue::NULL);
            }
            the_result => {
                log(format!("ABOUT TO PANIC! Unimplemented async result: {:?}", the_result).as_str());
                todo!("Not implemented async result");
                //browser_request_animation_frame(f.borrow().as_ref().unwrap());
            }
        }
    }
    
    
    // fn step_program(&mut self, last_step: &mut WsStepExecutionInfo)
    //                     -> WsStepExecutionInfo {
    //     let mut mut_context = self.current_execution_context.as_mut().unwrap();
    //     let tmp_result =
    //         self.program.step(last_step.get_inner_execution(), mut_context);
    //     WsStepExecutionInfo::new(tmp_result, false)
    // }
    
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
                parser::ParserResult::Error(error) => {
                    HtmlDivConsole::new().print_line(&format!("Error: {}", error));
                }
                parser::ParserResult::Nothing => {
                    HtmlDivConsole::new().print_line("Nothing");
                }
            };
        } else {
            log("2. ???");
            match parser::parse_repl_instruction_string (uline) {
                parser::ParserResult::Success(parsed_instr) => {
                    log("about to eval instruction");
                    let mut context = eval::EvaluationContext::with_program(&mut self.program, Box::new(HtmlDivConsole::new()));
                    log("2. eval in interpreter__");
                    let result = parsed_instr.eval(-1,
                                      eval_arg,
                                      &mut context,
                                                   &mut self.program);
                    log("load async 1");
                    match result {
                        InstructionResult::RequestAsyncAction(async_action)  => {
                            return Some(async_action);
                        }
                        InstructionResult::EvaluateToError(ref message) => {
                           context.console.print_line(message)
                        }                       
                        _ => {}// todo!("Not implemented result of instruction")
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
