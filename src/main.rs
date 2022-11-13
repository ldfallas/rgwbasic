pub mod eval;
pub mod tokens;
pub mod parser;
use std::io::{self, BufRead};

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
        if uline.len() > 0 && uline.chars().nth(0).unwrap().is_digit(10) {
            match parser::parse_instruction_line_from_string(uline) {
                parser::ParserResult::Success(parsed_line) => program.add_line(parsed_line),
                parser::ParserResult::Error(error) => println!("Error: {}", error),
                parser::ParserResult::Nothing=> println!("Nothing")       
            }
        } else {
            match parser::parse_repl_instruction_string (uline) {
                parser::ParserResult::Success(parsed_instr) => {
                    let mut context = eval::EvaluationContext::with_program(&mut program);
                    parsed_instr.eval(-1, &mut context);
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
