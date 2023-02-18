pub mod binary;
pub mod context;
pub mod def_instr;
pub mod dim_instr;
pub mod for_instr;
pub mod if_instr;
pub mod print_using;
pub mod while_instr;

#[macro_use]
pub mod utils;
pub mod swap_instr;
pub mod data_instr;
pub mod gosub_instr;

pub use crate::eval::context::{
    evaluate_to_usize, EvaluationContext, ExpressionEvalResult, ExpressionType, GwInstruction,
    GwProgram, InstructionResult, LineExecutionArgument, ProgramLine,
};

pub type EvaluationError = String;

pub trait GwExpression {
    fn eval(&self, context: &mut EvaluationContext) -> Result<ExpressionEvalResult, EvaluationError>;
    fn fill_structure_string(&self, buffer: &mut String);
}

pub trait GwAssignableExpression: GwExpression {
    fn get_type(&self, context: &EvaluationContext) -> ExpressionType;
    fn assign_value(
        &self,
        value: ExpressionEvalResult,
        context: &mut EvaluationContext,
    ) -> Result<(), String>;
}

//  Node for function call or array access elements for example:
//  ```
//     FOO(123)
//     LOG(x)
//  ```
//
pub struct GwParenthesizedAccessExpr {
    name: String,
    arguments: Vec<Box<dyn GwExpression>>,
}

impl GwExpression for GwParenthesizedAccessExpr {
    fn eval(&self, context: &mut EvaluationContext)
               -> Result<ExpressionEvalResult, EvaluationError> {
        let mut evaluated_arguments = Vec::with_capacity(self.arguments.len());
        for arg in &self.arguments {
            match evaluate_to_usize(arg, context) {
                Ok(eval_index) => {
                    evaluated_arguments.push(eval_index);
                }
                Err(msg) => {
                    return Err(msg.to_string());
                }
            }
        }

        if let Some(array) = context.get_existing_array(&self.name) {
            Ok(array.get_value(evaluated_arguments))
        } else if let Some(_function) =
            context.get_existing_function(&self.name, self.arguments.len())
        {
            todo!();
        } else {
            Err("Subscript out of range".to_string())
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.name[..]);
        buffer.push_str("(");
        buffer.push_str(")");
    }
}

// AST node for special INKEY$ variable
pub struct GwInkey {
}

impl GwExpression for GwInkey {
    fn eval(&self, _context: &mut EvaluationContext) -> Result<ExpressionEvalResult, EvaluationError> {
        //TODO implement this!
        Ok(ExpressionEvalResult::StringResult("".into()))
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("INKEY$");
    }
}

// AST node for elements representing function calls
pub struct GwCall {
    pub array_or_function: String,
    pub arguments: Vec<Box<dyn GwExpression>>,
}

impl GwAssignableExpression for GwCall {
    fn get_type(&self, _context: &EvaluationContext) -> ExpressionType {
        //return context.get_variable_type(&self.array_or_function).unwrap();
        return ExpressionType::Single;
    }

    fn assign_value(
        &self,
        value: ExpressionEvalResult,
        context: &mut EvaluationContext,
    ) -> Result<(), String> {
        let mut indices: Vec<usize> = vec![];
        for expr in &self.arguments {
            match evaluate_to_usize(expr, context) {
                Ok(index) if index > 0 => {
                    indices.push(index);
                }
                Ok(_) => {
                    return Err("Invalid index".to_string());
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        context.set_array_entry(&self.array_or_function, indices, &value);
        Ok(())
    }
}

impl GwExpression for GwCall {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        let mut indices: Vec<usize> = vec![];
        for expr in &self.arguments {
            match evaluate_to_usize(expr, context) {
                Ok(index)  => {
                    indices.push(index);
                }
                _ => {
                    todo!();
                }
            }
        }
        if let Some(array) = context.get_existing_array(&self.array_or_function) {
            Ok(array.get_value(indices))
        } else {
            todo!();
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.array_or_function[..]);
        buffer.push_str("(");
        let mut i = 0;
        for arg in &self.arguments {
            arg.fill_structure_string(buffer);
            if i != &self.arguments.len() - 1 {
                buffer.push(',')
            }
            i = i + 1
        }
        buffer.push_str(")");
    }
}

pub struct GwAbs {
    pub expr: Box<dyn GwExpression>,
}

impl GwExpression for GwAbs {
    fn eval(&self, context: &mut EvaluationContext) -> Result<ExpressionEvalResult, EvaluationError> {
        match self.expr.eval(context) {
            Ok(ExpressionEvalResult::IntegerResult(value)) => {
                Ok(ExpressionEvalResult::IntegerResult(value.abs()))
            }
            Ok(ExpressionEvalResult::SingleResult(value)) => {
                Ok(ExpressionEvalResult::SingleResult(value.abs()))
            }
            Ok(ExpressionEvalResult::DoubleResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult(value.abs()))
            }
            Ok(_) => {
                Err("Type mismatch".to_string())
            },
            err@Err(_) => err
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("ABS(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }
}

pub struct GwLog {
    pub expr: Box<dyn GwExpression>,
}

impl GwExpression for GwLog {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        match self.expr.eval(context) {
            Ok(ExpressionEvalResult::IntegerResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult((value as f64).ln()))
            }
            Ok(ExpressionEvalResult::SingleResult(value)) => {
                Ok(ExpressionEvalResult::SingleResult(value.ln()))
            }
            Ok(ExpressionEvalResult::DoubleResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult(value.ln()))
            }
            Ok(_) => Err("Type mismatch".to_string()),
            error@Err(_) => {
                error
            }
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("LOG(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }
}


pub struct GwSin {
    pub expr: Box<dyn GwExpression>,
}

impl GwExpression for GwSin {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        match self.expr.eval(context) {
            Ok(ExpressionEvalResult::IntegerResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult((value as f64).sin()))
            }
            Ok(ExpressionEvalResult::SingleResult(value)) => {
                Ok(ExpressionEvalResult::SingleResult(value.sin()))
            }
            Ok(ExpressionEvalResult::DoubleResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult(value.sin()))
            }
            Ok(_) => Err("Type mismatch".to_string()),
            error@Err(_) => {
                error
            }
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("SIN(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }
}

pub struct GwCos {
    pub expr: Box<dyn GwExpression>,
}

impl GwExpression for GwCos {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        match self.expr.eval(context) {
            Ok(ExpressionEvalResult::IntegerResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult((value as f64).cos()))
            }
            Ok(ExpressionEvalResult::SingleResult(value)) => {
                Ok(ExpressionEvalResult::SingleResult(value.cos()))
            }
            Ok(ExpressionEvalResult::DoubleResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult(value.cos()))
            }
            Ok(_) => Err("Type mismatch".to_string()),
            error@Err(_) => {
                error
            }
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("COS(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }
}

pub struct GwInt {
    pub expr: Box<dyn GwExpression>,
}

impl GwExpression for GwInt {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        match self.expr.eval(context) {
            full @ Ok(ExpressionEvalResult::IntegerResult(_)) => full,
            Ok(ExpressionEvalResult::DoubleResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult(value.trunc()))
            }
            Ok(_) => Err("Type mismatch".to_string()),
            error@Err(_) => {
                error
            }
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("INT(");
        self.expr.fill_structure_string(buffer);
        buffer.push(')')
    }
}

pub struct GwNegExpr {
    pub expr: Box<dyn GwExpression>,
}

impl GwExpression for GwNegExpr {
    fn eval(&self, context: &mut EvaluationContext)
            -> Result<ExpressionEvalResult, EvaluationError> {
        match self.expr.eval(context) {
            Ok(ExpressionEvalResult::IntegerResult(value)) => {
                Ok(ExpressionEvalResult::IntegerResult(-1 * value))
            }
            Ok(ExpressionEvalResult::DoubleResult(value)) => {
                Ok(ExpressionEvalResult::DoubleResult(-1.0 * value))
            }
            Ok(ExpressionEvalResult::SingleResult(value)) => {
                Ok(ExpressionEvalResult::SingleResult(-1.0 * value))
            }
            Ok(_) => Err("Type mismatch".to_string()),
            error@Err(_) => {
                error
            }
        }
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("-");
        self.expr.fill_structure_string(buffer);
    }
}

pub struct GwParenthesizedExpr {
    expr: Box<dyn GwExpression>,
}

impl GwParenthesizedExpr {
    pub fn new(expr: Box<dyn GwExpression>) -> GwParenthesizedExpr {
        GwParenthesizedExpr { expr: expr }
    }
}

impl GwExpression for GwParenthesizedExpr {
    fn eval(&self, context: &mut EvaluationContext) -> Result<ExpressionEvalResult,String> {
        return self.expr.eval(context);
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("(");
        self.expr.fill_structure_string(buffer);
        buffer.push_str(")");
    }
}

pub struct GwStringLiteral {
    value: String,
}

impl GwStringLiteral {
    pub fn with_value(value: String) -> GwStringLiteral {
        GwStringLiteral { value }
    }
}

impl GwExpression for GwStringLiteral {
    fn eval(&self, _context: &mut EvaluationContext)
                 -> Result<ExpressionEvalResult, EvaluationError> {
        Ok(ExpressionEvalResult::StringResult(String::from(&self.value)))
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.value[..]);
    }
}

pub struct GwIntegerLiteral {
    value: i16,
}

impl GwIntegerLiteral {
    pub fn with_value(value: i16) -> GwIntegerLiteral {
        GwIntegerLiteral { value }
    }
}

impl GwExpression for GwIntegerLiteral {
    fn eval(&self, _context: &mut EvaluationContext)
                 -> Result<ExpressionEvalResult, EvaluationError> {
        Ok(ExpressionEvalResult::IntegerResult(self.value))
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.value.to_string());
    }
}

pub struct GwDoubleLiteral {
    value: f64,
}

impl GwDoubleLiteral {
    pub fn with_value(value: f64) -> GwDoubleLiteral {
        GwDoubleLiteral { value }
    }
}

impl GwExpression for GwDoubleLiteral {
    fn eval(&self, _context: &mut EvaluationContext)
                     -> Result<ExpressionEvalResult, EvaluationError> {
        Ok(ExpressionEvalResult::DoubleResult(self.value))
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.value.to_string());
    }
}

pub struct GwVariableExpression {
    name: String,
}

impl GwAssignableExpression for GwVariableExpression {
    fn get_type(&self, context: &EvaluationContext) -> ExpressionType {
        context
            .get_variable_type(&self.name)
            .unwrap_or(ExpressionType::Double)
    }

    fn assign_value(&self,
                    value: ExpressionEvalResult,
                    context: &mut EvaluationContext)
               -> Result<(), String> {
        match context.set_variable(&self.name, &value) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }
}

impl GwVariableExpression {
    pub fn with_name(name: String) -> GwVariableExpression {
        GwVariableExpression { name }
    }
}

impl GwExpression for GwVariableExpression {
    fn eval(&self, context: &mut EvaluationContext)
                    -> Result<ExpressionEvalResult, EvaluationError> {
        if let Some(value) = context.lookup_variable(&self.name) {
            Ok(value.clone())
        } else {
            // TODO we need to define a variable here???
            Ok(ExpressionEvalResult::IntegerResult(0))
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.name[..]);
    }
}

impl ProgramLine {
    fn get_line(&self) -> i16 {
        self.line
    }

    // fn eval (&self, context : &mut EvaluationContext) -> InstructionResult {
    //      self.instruction.eval(self.line, context)
    // }

    pub fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push('(');
        buffer.push_str(&self.line.to_string()[..]);
        buffer.push(' ');
        self.instruction.fill_structure_string(buffer);
        if let Some(rest) = &self.rest_instructions {
            buffer.push(' ');
            for e in rest {
                buffer.push(':');
                e.fill_structure_string(buffer);
            }
        }
        buffer.push(')');
    }
}

pub struct GwListStat {}

impl GwInstruction for GwListStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,        
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        program.list(&mut context.console);
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"LIST");
    }
}

pub struct GwLoadStat {
    pub filename: Box<dyn GwExpression>,
}

impl GwInstruction for GwLoadStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,        
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        let result = self.filename.eval(context);

        return match result {
            Ok(ExpressionEvalResult::StringResult(filename)) => {
                match program.load_from(&filename.trim_matches('"'), &context.console) {
                    Ok(_) => {
                        println!("File loaded");
                    }
                    Err(error) => {
                        panic!("Error loading file {:?}", error);
                    }
                }
                InstructionResult::EvaluateNext
            }
            Ok(_) => {
                InstructionResult::EvaluateToError("Type mismatch".to_string())
            }
            Err(error) => {
                InstructionResult::EvaluateToError(error)
            }
        };
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"LOAD ");
        self.filename.fill_structure_string(buffer);
    }
}

pub struct GwRunStat {}

impl GwInstruction for GwRunStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
//        if let Some(program) = &context.underlying_program {
//            program.run(&context.console);
        //        }
        program.run(&context.console);
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"RUN");
    }
}

pub struct GwSystemStat {}

impl GwInstruction for GwSystemStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        context.console.exit_program();
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"SYSTEM");
    }
}


pub struct GwRem {
    pub comment: String,
}

impl GwInstruction for GwRem {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        _context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"REM ");
        buffer.push_str(&self.comment[..]);
    }
}

pub struct GwCls {}

impl GwInstruction for GwCls {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        _context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"CLS");
    }
}

pub struct GwAssign {
    pub variable: String,
    pub expression: Box<dyn GwExpression>,
}

impl GwInstruction for GwAssign {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {

        //let expression_evaluation = ;
        match self.expression.eval(context) {
            Ok(expression_evaluation) =>{
                if let None = context.get_variable_type(&self.variable) {
                    context.set_variable_type(&self.variable, &ExpressionType::Single);
                }
                match context.set_variable(&self.variable, &expression_evaluation) {
                    Err(error_message)
                        => InstructionResult::EvaluateToError(error_message.to_string()),
                    _ => InstructionResult::EvaluateNext
                }
            }
            Err(error) => InstructionResult::EvaluateToError(error)
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.variable[..]);
        buffer.push_str(&" = ");
        self.expression.fill_structure_string(buffer);
    }
}

//

pub struct GwArrayAssign {
    pub variable: String,
    pub indices_expressions: Vec<Box<dyn GwExpression>>,
    pub expression: Box<dyn GwExpression>,
}

impl GwInstruction for GwArrayAssign {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        let mut evaluated_arguments: Vec<usize> =
                   Vec::with_capacity(self.indices_expressions.len());
        for arg in &self.indices_expressions {
            match evaluate_to_usize(arg, context) {
                Ok(index) => {
                    evaluated_arguments.push(index);
                }
                Err(error) => {
                    return InstructionResult::EvaluateToError(error);
                }
            }
        }

        match self.expression.eval(context) {
            Ok(expression_evaluation) => {
                context.set_array_entry(
                    &self.variable,
                    evaluated_arguments,
                    &expression_evaluation,
                );
                InstructionResult::EvaluateNext
            }
            Err(error) => InstructionResult::EvaluateToError(error),
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.variable[..]);
        buffer.push_str(&"(");
        for arg in &self.indices_expressions {
            arg.fill_structure_string(buffer);
        }

        buffer.push_str(&") = ");
        self.expression.fill_structure_string(buffer);
    }
}

//

pub struct GwGotoStat {
    pub line: i16,
}

impl GwInstruction for GwGotoStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        if let Some(actual_line) = context.get_real_line(self.line) {
            return InstructionResult::EvaluateLine(actual_line);
        } else {
            println!("-- {}", self.line);
            panic!("Trying to jump to non existing line");
        }
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("GOTO ");
    }
}

pub enum SwitchIndicator {
    On,
    Off,
}

pub struct GwKeyStat {
    pub indicator: SwitchIndicator,
}

impl GwInstruction for GwKeyStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        _context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"KEY ");
        match self.indicator {
            SwitchIndicator::On => buffer.push_str("ON"),
            SwitchIndicator::Off => buffer.push_str("OFF"),
        }
    }
}

pub struct GwColor {
    pub red: Box<dyn GwExpression>,
    pub green: Box<dyn GwExpression>,
    pub blue: Box<dyn GwExpression>,
}

impl GwInstruction for GwColor {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        _context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        InstructionResult::EvaluateNext
    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"COLOR ");
        self.red.fill_structure_string(buffer);
        buffer.push_str(&", ");
        self.green.fill_structure_string(buffer);
        buffer.push_str(&", ");
        self.blue.fill_structure_string(buffer);
    }
}

pub enum PrintSeparator {
    Comma,
    Semicolon,
}

pub enum PrintElementWrapper {
    Nothing,
    Expr(Box<dyn GwExpression>),
    Tab(Box<dyn GwExpression>),
}

pub struct GwPrintStat {
    pub expressions: Vec<(PrintElementWrapper, Option<PrintSeparator>)>,
}

impl GwInstruction for GwPrintStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        let mut i = 0;
        let mut newline_at_the_end = true;
        for print_expr in &self.expressions {
            match print_expr {
                (PrintElementWrapper::Expr(expr), separator) => {
                    //let evaluated_expr = expr.eval(context);
                    match expr.eval(context) {
                        Ok(evaluated_expr) => {
                            context.console.print(&evaluated_expr.to_string());
                            if i == &self.expressions.len() - 1 {
                                if let Some(PrintSeparator::Semicolon) = separator {
                                    newline_at_the_end = false;
                                }
                            }
                        }
                        Err(eval_error) => {
                            return InstructionResult::EvaluateToError(eval_error);
                        }
                    }
                }
                (PrintElementWrapper::Tab(position_expr), _) => {
                    match evaluate_to_usize(position_expr, context) {
                        Ok(position) => {
                            context.console.adjust_to_position(position as usize);
                        }
                        Err(error) => {
                            return InstructionResult::EvaluateToError(error);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }

        let console = &mut context.console;
        if newline_at_the_end {
            console.print_line("");
        } else {
            console.print("");
        }
        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"PRINT ");

        for print_expr in &self.expressions {
            match print_expr {
                (PrintElementWrapper::Expr(expr), _) => {
                    expr.fill_structure_string(buffer);
                }
                (PrintElementWrapper::Tab(position), _) => {
                    buffer.push_str("TAB(");
                    position.fill_structure_string(buffer);
                    buffer.push_str(")");
                }
                _ => {}
            }
            buffer.push_str(";");
        }
    }
}

pub struct GwInputStat {
    pub prompt: Option<String>,
    pub variables: Vec<Box<dyn GwAssignableExpression>>,
}

fn read_variable_from_input(
    variable: &Box<dyn GwAssignableExpression>,
    context: &mut EvaluationContext,
    str_value: &str,
) -> Result<(), String> {
    match variable.get_type(context) {
        ExpressionType::Double => {
            let dbl = str_value.trim_end().parse::<f64>().unwrap();
            variable.assign_value(ExpressionEvalResult::DoubleResult(dbl), context)
        }
        ExpressionType::Single => {
            let svl = str_value.trim_end().parse::<f32>().unwrap();
            variable.assign_value(ExpressionEvalResult::SingleResult(svl), context)
        }
        _ => panic!("Not implemented INPUT for this type"),
    }
}

impl GwInstruction for GwInputStat {
    fn eval(
        &self,
        _line: i16,
        _arg: LineExecutionArgument,
        context: &mut EvaluationContext,
        program: &mut GwProgram
    ) -> InstructionResult {
        let mut buffer = String::new();
        let mut pr = "?";
        if let Some(ref prompt) = self.prompt {
            pr = prompt.as_str();
        }

        context.console.print(pr);
        context.console.read_line(&mut buffer);

        let mut var_idx = 0;
        for part in buffer.split(',') {
            let read_result =
                read_variable_from_input(&self.variables[var_idx], context, part);
            if let Err(error_message) = read_result {
                return InstructionResult::EvaluateToError(error_message);
            }
            var_idx = var_idx + 1;
        }

        InstructionResult::EvaluateNext
    }

    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&"INPUT ");
        match &(self.prompt) {
            Some(ptp) => {
                buffer.push_str("\"");
                buffer.push_str(ptp.as_str());
                buffer.push_str("\",");
            }
            _ => {}
        }
        let mut i = 0;
        for variable in &self.variables {
            (*variable).fill_structure_string(buffer);
            if i != self.variables.len() - 1 {
                buffer.push_str(",");
            }
            i = i + 1;
        }
    }
}

#[cfg(test)]
mod eval_tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::eval::ExpressionEvalResult;
    use crate::eval::context::Console;
    use crate::eval::*;

    #[test]
    fn it_tests_basic_eval() {
        let line1 = ProgramLine {
            line: 10,
            instruction: Rc::new(GwAssign {
                variable: String::from("X"),
                expression: Box::new(GwIntegerLiteral { value: 10 }),
            }),
            rest_instructions: None,
        };
        let clonned_instr = (&line1.instruction).clone();

        let mut program = GwProgram {
            lines: vec![line1],
            data: vec![],
            real_lines: vec![clonned_instr]
        };

        let mut context = EvaluationContext {
            variables: HashMap::new(),
            array_variables: HashMap::new(),
            jump_table: HashMap::new(),
    //        underlying_program: None,
            pair_instruction_table: HashMap::new(),
  //          real_lines: Some(vec![&program.lines.get(0).unwrap().instruction]),
            console: Box::new(DummyConsole{}),
//            data: vec![],
            data_position: -1,
            subroutine_stack: vec![],
            current_real_line: -1
        };

        context
            .variables
            .insert(String::from("X"), ExpressionEvalResult::IntegerResult(5));

        if let Some(ExpressionEvalResult::IntegerResult(value)) =
            context.lookup_variable(&String::from("X"))
        {
            let some_value: i16 = 5;
            assert_eq!(&some_value, value);
        }

        program.eval(&mut context);

        if let Some(ExpressionEvalResult::IntegerResult(value)) =
            context.lookup_variable(&String::from("X"))
        {
            let some_value: i16 = 10;
            assert_eq!(&some_value, value);
        }
    }

    #[test]
    fn it_negates_integer_expressions() -> Result<(), & 'static str> {
        let negation = GwNegExpr {
            expr: Box::new(GwIntegerLiteral::with_value(1)),
        };

        let mut context = empty_context();

        match negation.eval(&mut context) {
            Ok(ExpressionEvalResult::IntegerResult(x)) => {
                assert_eq!(x, -1);
                Ok(())
            }
            _ => Err("Negation not evaluated as expected")
        }
    }

    #[test]
    fn it_executes_array_access() -> Result<(), String> {
        let array_access = GwCall {
            array_or_function: "arr".to_string(),
            arguments: vec![Box::new(GwIntegerLiteral::with_value(2))],
        };
        let mut context = empty_context();
        context.declare_array("arr", 5);
        context.set_array_entry(
            "arr",
            vec![2 as usize],
            &ExpressionEvalResult::IntegerResult(101),
        );

        match array_access.eval(&mut context) {
            Ok(ExpressionEvalResult::IntegerResult(101)) => Ok(()),
            _ => Err("Invalid array access result".to_string()),
        }
    }

    #[test]
    fn it_negates_double_expressions() -> Result<(), & 'static str>{
        let negation = GwNegExpr {
            expr: Box::new(GwDoubleLiteral::with_value(2.5)),
        };

        let mut context = empty_context();

        match negation.eval(&mut context) {
            Ok(ExpressionEvalResult::DoubleResult(x)) => {
                assert_eq!(x, -2.5);
                Ok(())
            }
            _ => Err("Negation not evaluated")
        }
    }

    #[test]
    fn it_tests_basic_array_eval() {
        let line1 = ProgramLine {
            line: 10,
            instruction: Rc::new(GwArrayAssign {
                variable: String::from("A"),
                indices_expressions: vec![Box::new(GwIntegerLiteral::with_value(1))],
                expression: Box::new(GwIntegerLiteral { value: 12 }),
            }),
            rest_instructions: None,
        };

        let mut program = GwProgram { lines: vec![line1], data: vec![], real_lines: vec![] };

        let mut context = EvaluationContext::new(Box::new(DummyConsole{}));
//        context.real_lines = Some(vec![]);

        context.declare_array("A", 10);

        let arr1 = context.get_existing_array("A");

        if let ExpressionEvalResult::IntegerResult(value) = arr1.unwrap().get_value(vec![1]) {
            let some_value: i16 = 0;
            assert_eq!(some_value, value);
        }

        program.eval(&mut context);

        let arr2 = context.get_existing_array("A");

        if let ExpressionEvalResult::IntegerResult(value) = arr2.unwrap().get_value(vec![1]) {
            let some_value: i16 = 12;
            assert_eq!(some_value, value);
        }
    }

    pub fn empty_program() -> GwProgram {
        GwProgram {
            lines: vec![],
            real_lines: vec![],
            data: vec![],
        }
    }

    pub fn empty_context() -> EvaluationContext {
        EvaluationContext {
            variables: HashMap::new(),
            array_variables: HashMap::new(),
            jump_table: HashMap::new(),
            //underlying_program: None,
            pair_instruction_table: HashMap::new(),
            //real_lines: None,
            console: Box::new(DummyConsole{}),
            //data: vec![],
            data_position: -1,
            subroutine_stack: vec![],
            current_real_line: -1
        }
    }

    pub struct DummyConsole {
    }

    impl Console for DummyConsole {
        fn print(&mut self, value: &str) {
        todo!()
    }

        fn print_line(&mut self, value: &str) {
        todo!()
    }

        fn read_line(&mut self, buffer: &mut String) {
        todo!()
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
        todo!()
    }

        fn exit_program(&self) {
        todo!()
    }

        fn clone(&self) -> Box<dyn Console> {
        todo!()
    }
    }
}
