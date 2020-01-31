
pub mod tokens;
pub mod parser;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    println!("Start");

    for line in io::stdin().lock().lines() {
        let uline = line.unwrap();
        if uline.eq_ignore_ascii_case("system") {
            break;
        }
        
        match parser::parse_instruction_line_from_string(uline) {
            parser::ParserResult::Success(_) => println!("Parsed"),
            parser::ParserResult::Error(error) => println!("Error: {}", error),
            parser::ParserResult::Nothing=> println!("Nothing")            
        }
    }
    
    Ok(())
}
