
pub mod tokens;
pub mod parser;
use std::io::{self, BufRead};

fn read_stdin_line(line : &mut String) -> bool {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut tmp_line = String::new();
    match handle.read_line(&mut tmp_line) {
        Ok(_) => {
            line.push_str(tmp_line.trim_end());
            true
        },
        Err(_) => false
    }
}

fn main() -> io::Result<()> {
    let mut program = parser::GwProgram::new();
    println!("Ok");
    //for line in stdinLock.lines() {
    let mut uline = String::new();
    let mut success = read_stdin_line(&mut uline);
    while success {

//        let uline = line.unwrap();
        if uline.eq_ignore_ascii_case("system") {
            break;
        } else if uline.eq_ignore_ascii_case("list") {
           program.list();
        } else if uline.eq_ignore_ascii_case("run") {
           program.run();
           println!("Ok");
        } else {
        
           match parser::parse_instruction_line_from_string(uline) {
               parser::ParserResult::Success(parsed_line) => program.add_line(parsed_line),
               parser::ParserResult::Error(error) => println!("Error: {}", error),
               parser::ParserResult::Nothing=> println!("Nothing")       

           }
        }
        uline = String::new();
        success = read_stdin_line(&mut uline)                

    }
    
    Ok(())
}
