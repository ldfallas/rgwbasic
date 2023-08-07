pub mod eval;
pub mod tokens;
pub mod parser;

#[cfg(test)]
mod integration_tests {
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::eval::context::Console;
    use super::eval::*;

    #[test]
    fn it_should_run_builtins() -> Result<(), String> {
        let code = "\
10 PRINT USING \"###.##\"; LOG(2.7)
20 PRINT USING \"###.##\"; COS(0)
30 PRINT USING \"###.##\"; SIN(3.14)
40 PRINT LEFT$(\"HELLO\", 3)";
        let mut program = GwProgram::new();
        let rc_str = Rc::new(RefCell::new(String::new()));
        let mut console: Box<dyn Console> = Box::new(TestConsole::new(rc_str.clone()));
        let file_lines = Box::new(code.split("\n").map(|s| s.to_string()));
        let _ = program.load_from(&mut console, file_lines);

        program.run(&console);

        let str_borrow = rc_str.borrow();
        let mut console_it = str_borrow.split('\n');
        
        assert_eq!(Some("  0.99"), console_it.next());
        assert_eq!(Some("  1.00"), console_it.next());
        assert_eq!(Some("  0.00"), console_it.next());
        assert_eq!(Some("HEL"), console_it.next());
        
        Ok(())
    }


    pub struct TestConsole {
        contents: Rc<RefCell<String>>
    }
    
    impl TestConsole {
        pub fn new(string_ref: Rc<RefCell<String>>) -> TestConsole {
            TestConsole { contents: string_ref }
        }
    }

    impl Console for TestConsole {
        fn print(&mut self, value: &str) {
            let mut m = self.contents.borrow_mut();
            m.push_str(value);
        }

        fn print_line(&mut self, value: &str) {
            let mut m = self.contents.borrow_mut();
            m.push_str(value);
            m.push_str("\n");
        }

        fn read_line(&mut self, _buffer: &mut String) {
            todo!()
        }
        
        fn clear_screen(&mut self) {
            todo!()
        }

        fn current_text_column(&self) -> usize {
            todo!()
        }

        fn read_file_lines(&self, _file_name: &str) -> Box<dyn Iterator<Item=String>> {
            todo!()
        }

        fn flush(&self) {
            todo!()
        }

        fn exit_program(&self) {
            todo!()
        }

        fn clone(&self) -> Box<dyn Console> {
            Box::new(TestConsole::new(self.contents.clone()))
        }
    }
}
