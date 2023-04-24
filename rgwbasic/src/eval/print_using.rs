use crate::eval::ExpressionEvalResult;

use super::{
    EvaluationContext,
    InstructionResult,
    GwInstruction,
    GwProgram,
    LineExecutionArgument,
    PrintSeparator,
    PrintElementWrapper
};

pub struct GwPrintUsingStat {
    pub expressions : Vec<(PrintElementWrapper, Option<PrintSeparator>)>
}

impl GwPrintUsingStat {
    pub fn print_formatted_string(&self, format_string: &str, context: &mut EvaluationContext) -> InstructionResult {
        let mut arg_i = 1;
        loop {
            let mut tmp_format = &format_string[..];
            loop {
                match tok_format_string(tmp_format) {
                    PrintUsingFormatFragment::Literal(literal, rest) => {
                        context.console.print(literal);
                        tmp_format = rest;
                    }
                    PrintUsingFormatFragment::Numeric { dollar, digits, comma, decimals, rest} => {
                        //tmp_format = rest;
                        let  value_to_use: f64;
                        if let Some((PrintElementWrapper::Expr(arg), _)) = self.expressions.get(arg_i) {
                            match arg.eval(context) {
                                Ok(ExpressionEvalResult::DoubleResult(dbl)) => {
                                    value_to_use  = dbl;
                                }
                                Ok(ExpressionEvalResult::SingleResult(dbl)) => {
                                    value_to_use  = dbl as f64;
                                }

                                Ok(ExpressionEvalResult::IntegerResult(ival)) => {
                                    value_to_use  = ival as f64;
                                }
                                _ => {
                                    return InstructionResult::EvaluateToError(String::from("Invalid value"))
                                }
                            }
                        } else {
                            todo!();
                        }

                        let mut format_buf = String::new();
                        format_number(value_to_use,
                                      dollar,
                                      digits,
                                      comma,
                                      decimals,
                                      &mut format_buf);
                        context.console.print(format_buf.as_str());
                        tmp_format = rest;
                        arg_i += 1;
                    },
                    PrintUsingFormatFragment::End(last) => {
                        context.console.print(last);
                        break;
                    }
                }
            }
            if arg_i == self.expressions.len() && arg_i != 1 {
                break;
            }
        }
        match self.expressions.last() {
            Some((_, Some(PrintSeparator::Semicolon))) => {
            }
            _ => { context.console.print_line(""); }
        }
        InstructionResult::EvaluateNext
    }

}

impl GwInstruction for GwPrintUsingStat {
    fn eval (&self,
             _line: i16,
             _arg: LineExecutionArgument,
             context: &mut EvaluationContext,
             _program: &mut GwProgram) -> InstructionResult {
        if let Some((PrintElementWrapper::Expr(expr), _)) = self.expressions.get(0) {
            if let Ok(ExpressionEvalResult::StringResult(a_atr)) = expr.eval(context) {
                let format_string = &a_atr.as_str();
                return self.print_formatted_string(format_string, context);
            }
            else {
                return InstructionResult::EvaluateToError(String::from("Invalid function call"))
            }
        } else {
            return InstructionResult::EvaluateToError(String::from("Invalid function call"))
        }
    }

    fn fill_structure_string(&self, buffer : &mut String) {
        buffer.push_str("PRINT USING ");
    }
}

pub fn format_number(
    value: f64,
    dollar: bool,
    digits:i8,
    comma: bool,
    decimals: i8,
    target: &mut String) {
//    let mut i = 0;
    let mut tmp = (value as i32).abs();
    target.clear();
    if decimals > 0 {
        let tenspow = (10 as i32).pow(decimals as u32);
        let mut start = (((value - value.trunc())  * (tenspow as f64)) as i32).abs();
        let mut idec = decimals;
        while idec > 0 {
            let digit = (start % 10) as u32;
            target.push(char::from_digit(digit , 10).unwrap());
            start /= 10;
            idec -= 1;
          //  i += 1;
        }
        target.push('.');
        //i += 1;
    }
    let mut idigits = digits;
    let mut i = 0;
    while idigits > 0 {
        if comma
            && i > 0
            && i % 3 == 0
            && tmp > 0 {
            target.push(',');
        }
        if tmp > 0 {
            let digit = tmp % 10;
            target.push(char::from_digit(digit as u32 , 10).unwrap());
            tmp /= 10;
            if tmp == 0 &&  dollar {
                target.push('$');
            }
        } else {
            target.push(' ');
        }
        idigits -= 1;
        i += 1;
    }
    if value < 0.0 {
        target.push('-');
    }


    // TODO there must be a better way without unsafe to do this:
    let new_result:String = target.chars().rev().collect();
    target.clear();
    target.insert_str(0, new_result.as_str());
}

pub enum PrintUsingFormatFragment<'a> {
    Literal(&'a str, &'a str),
    Numeric { dollar: bool, digits:i8, comma: bool, decimals: i8, rest: &'a str },
    End(&'a str)
}

fn tok_numeric_format(input: &str) -> PrintUsingFormatFragment {
    let mut dollar = false;
    let mut digits = 0;
    let mut after_dot = false;
    let mut comma = false;
    let mut decimals = 0;
    let mut i = 0;
    for c in input.chars() {
        if c == '$' {
            dollar = true;
        }
        else if c == '#' && !after_dot {
            digits += 1;
        }
        else if c == '#' && after_dot {
            decimals += 1;
        }
        else if c == '.' {
            after_dot = true;
        }
        else if c == ',' {
            comma = true;
        } else {
            break;
        }
        i += c.len_utf8();
    }
    return PrintUsingFormatFragment::Numeric {
        dollar, digits, comma, decimals, rest: &input[i..]
    };
}

pub fn tok_format_string<'a>(input: &'a str) -> PrintUsingFormatFragment<'a> {
    let mut i = 0;
    for c in input.chars() {
        if c == '#' || c == '$'  {
            if i > 0 {
                return PrintUsingFormatFragment::Literal(&input[0..i], &input[i..])
            } else {
                return tok_numeric_format(input);
            }
        }
        i += c.len_utf8();
    }
    PrintUsingFormatFragment::End(input)
}

#[cfg(test)]
mod print_using_tests {
    use crate::eval::print_using::*;

    #[test]
    fn it_process_string_without_format() {
        let result = tok_format_string("This is a test");
        if let PrintUsingFormatFragment::End(the_str) = result {
            assert_eq!("This is a test", the_str);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn it_process_string_without_format_and_whitespace() {
        let result = tok_format_string("  %");
        if let PrintUsingFormatFragment::End(the_str) = result {
            assert_eq!("  %", the_str);
        } else {
            assert!(false);
        }
    }


    #[test]
    fn it_tokenize_only_format_string() {
        let mut result = tok_format_string("###.##");
        if let PrintUsingFormatFragment::Numeric{ rest, .. } = result {
            assert_eq!("", rest);
            result = tok_format_string(rest);
            if let PrintUsingFormatFragment::End(last) = result {
                assert_eq!("", last);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn it_tokenize_only_format_string_with_currency() -> Result<(), & 'static str> {
        let mut result = tok_format_string("$$###,###,###.##");
        if let PrintUsingFormatFragment::Numeric {
                dollar:true,
                digits: 9,
                comma: true,
                decimals: 2,
                rest
            } = result {
            assert_eq!("", rest);
            result = tok_format_string(rest);
            if let PrintUsingFormatFragment::End(_) = result {
                Ok(())
            } else {
                Err("Format not recognized")
            }
        } else {
            Err("Format not recognized")
        }
    }

    #[test]
    fn it_process_string_with_single_number_format() {
        let result = tok_format_string("This is ### a test");
        if let PrintUsingFormatFragment::Literal(literal, rest1) = result {
            assert_eq!("This is ", literal);
            assert_eq!("### a test", rest1);

            if let PrintUsingFormatFragment::Numeric{
                dollar:false,
                digits: 3,
                comma: false,
                decimals: 0,
                rest: rest2
            } = tok_format_string(rest1) {
                assert_eq!(" a test", rest2);
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn it_process_string_with_numeric_at_start() -> Result<(), & 'static str> {
        let result = tok_format_string("####.#### %");
        if let PrintUsingFormatFragment::Numeric{
                dollar:false,
                digits: 4,
                comma: false,
                decimals: 4,
                rest
            } = result {
            assert_eq!(" %", rest);
            Ok(())
        } else {
            Err("Format not recognized")
        }
    }

    #[test]
    fn it_formats_simple_num() {
        let mut the_string = String::new();
        format_number(482.245,
                      false,
                      4,
                      false,
                      2,
                      &mut the_string);
        assert_eq!(" 482.24", the_string);
    }

    #[test]
    fn it_formats_simple_negative_num() {
        let mut the_string = String::new();
        format_number(-482.245,
                      false,
                      3,
                      false,
                      2,
                      &mut the_string);
        assert_eq!("-482.24", the_string);
    }

    #[test]
    fn it_formats_with_currency() {
        let mut the_string = String::new();
        format_number(27749.479,
                      true,
                      9,
                      true,
                      2,
                      &mut the_string);
        assert_eq!("    $27,749.47", the_string);
    }

    #[test]
    fn it_formats_simple_num_with_comma() {
        let mut the_string = String::new();
        format_number(4331.245,
                      false,
                      6,
                      true,
                      2,
                      &mut the_string);
        assert_eq!("  4,331.24", the_string);
    }
}
