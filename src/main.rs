
pub mod tokens;
pub mod parser;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    println!("Start");
    let mut program = parser::GwProgram::new();

    for line in io::stdin().lock().lines() {
        let uline = line.unwrap();
        if uline.eq_ignore_ascii_case("system") {
            break;
        }

        if uline.eq_ignore_ascii_case("list") {
           program.list();
           continue;
        }

        
        if uline.eq_ignore_ascii_case("run") {
           program.run();
           continue;
        }
        
        match parser::parse_instruction_line_from_string(uline) {
            parser::ParserResult::Success(parsed_line) => program.add_line(parsed_line),
            parser::ParserResult::Error(error) => println!("Error: {}", error),
            parser::ParserResult::Nothing=> println!("Nothing")            
        }
    }
    
    Ok(())
}
