use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::exit;
use rgwbasic::eval::context::Console;


pub struct DefaultConsole {
    column_position: usize
}

impl DefaultConsole {
    pub fn new() -> DefaultConsole {
        DefaultConsole {  column_position: 0 }
    }
}

impl Console for DefaultConsole {
    fn print(&mut self, value: &str) {
        print!("{}", value);
        self.column_position += value.len();
    }

    fn print_line(&mut self, value: &str) {
        println!("{}", value);
        self.column_position = 0;
    }

    fn read_file_lines(&self, file_name: &str) -> Box<dyn Iterator<Item=String>> {
        let f = File::open(file_name).unwrap();
        let reader = BufReader::new(f);
        Box::new(
            reader.lines().map(|line_result| line_result.unwrap()))
    }
    
    fn read_line(&mut self, buffer: &mut String) {
        io::stdout().flush().expect("Success");
        io::stdin().read_line(buffer).expect("Success");
    }
    fn clear_screen(&mut self) {
        todo!();
    }
    fn current_text_column(&self) -> usize{
        self.column_position + 1
    }
    fn flush(&self)  {
        io::stdout().flush().expect("Success");
    }
    fn exit_program(&self) {
        exit(0);
    }
    fn clone(&self) -> Box<dyn Console> {
        Box::new(DefaultConsole { column_position: self.column_position } )
    }
}
