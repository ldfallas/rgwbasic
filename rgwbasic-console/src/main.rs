use rgwbasic::{parser, eval};
use std::io::{self, BufRead};
mod defaultconsole;

use defaultconsole::DefaultConsole;

fn read_stdin_line(line : &mut String) -> bool {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut tmp_line = String::new();
    match handle.read_line(&mut tmp_line) {
        Ok(0) => false,
        Ok(_) => {
            line.push_str(tmp_line.trim_end());
            true
        },
        Err(_) => false
    }
}

fn main() -> io::Result<()> {
    let mut program = eval::GwProgram::new();
    println!("Ok");
    let mut uline = String::new();
    let mut success = read_stdin_line(&mut uline);
    while success {
        if !uline.is_empty() && uline.chars().next().unwrap().is_ascii_digit() {
            match parser::parse_instruction_line_from_string(uline) {
                parser::ParserResult::Success(parsed_line) => program.add_line(parsed_line),
                parser::ParserResult::Error(error) => println!("Error: {}", error),
                parser::ParserResult::Nothing=> println!("Nothing")       
            }
        } else {
            match parser::parse_repl_instruction_string (uline) {
                parser::ParserResult::Success(parsed_instr) => {
                    let mut context = eval::EvaluationContext::with_program(&mut program, Box::new(DefaultConsole::new()));
                    parsed_instr.eval(-1, eval::LineExecutionArgument::Empty, &mut context);
                    context.console.flush();
                }
                parser::ParserResult::Error(msg) => {
                    println!("Error parsing command {}", msg);
                }
                _ => {
                    println!("Error processing command");
                }
            }
            
        }
        uline = String::new();
        success = read_stdin_line(&mut uline);
    }
    
    Ok(())
}
