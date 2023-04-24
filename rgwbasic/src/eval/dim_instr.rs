use super::{EvaluationContext, 
            GwInstruction, GwExpression, GwProgram,
            evaluate_to_usize,
            InstructionResult,
            LineExecutionArgument};
use std::result::Result;

pub struct GwDimDecl {
    name: String,
    dimensions: Vec<Box<dyn GwExpression>>
}

impl GwDimDecl {
    pub fn new(name: String, dimensions: Vec<Box<dyn GwExpression>>) -> GwDimDecl {
        GwDimDecl {
            name,
            dimensions,
        }
    }
    fn perform_declaration(&self, context : &mut EvaluationContext) -> Result<(), String> {

        match evaluate_sequence_of_integers(&self.dimensions, context) {
            Ok(dimensions_to_use) if dimensions_to_use.len() > 0 => {
                context.declare_array(
                            &self.name,
                            *dimensions_to_use.get(0).unwrap() as usize);
                Ok(())
            }
            Ok(_) => Err("Dimensions are required".to_string()),
            Err(e) => Err(e)
        }
    }

    pub fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str(&self.name);
        buffer.push_str("(");
        append_to_string_with_separator(buffer,
                                        &self.dimensions,
                                         ",");
        buffer.push_str(")");
        
    }
}

pub struct GwDim {
    declaration: GwDimDecl,
    rest: Option<Vec<GwDimDecl>>
}

impl GwDim {
    pub fn new(declaration: GwDimDecl,
               rest: Option<Vec<GwDimDecl>>) -> GwDim {
        GwDim {
            declaration,
            rest
        }
    }
}

impl GwInstruction for GwDim {
    fn eval(&self,
             _line: i16,
             _arg: LineExecutionArgument,
            context : &mut EvaluationContext,
            _program: &mut GwProgram) -> InstructionResult {
        if let Err(e) = self.declaration.perform_declaration(context) {
            return InstructionResult::EvaluateToError(e);
        }
        if let Some(other_declarations) = &self.rest {
            for other_dim in other_declarations {
                if let Err(e) = other_dim.perform_declaration(context) {
                    return InstructionResult::EvaluateToError(e);
                }
            }
        }
        
        InstructionResult::EvaluateNext
        // match evaluate_sequence_of_integers(&self.dimensions, context) {
        //     Ok(dimensions_to_use) if dimensions_to_use.len() > 0 => {
        //         context.declare_array(&self.name, *dimensions_to_use.get(0).unwrap());
        //         InstructionResult::EvaluateNext
        //     }
        //     Ok(_) => { return InstructionResult::EvaluateToError("Dimensions are required".to_string())}
        //     Err(e) => { return InstructionResult::EvaluateToError(e) }
        // }

    }
    fn fill_structure_string(&self, buffer: &mut String) {
        buffer.push_str("(DIM ");

        self.declaration.fill_structure_string(buffer);

    }
}

fn append_to_string_with_separator(target: &mut String,
                                   values: &Vec<Box<dyn GwExpression>>,
                                   separator: &str) {
    let count = values.len();
    let mut i = 0;
    for e in values {
        e.fill_structure_string(target);
        i += 1;
        if i != count {
            target.push_str(separator);
        }
    }
}

fn evaluate_sequence_of_integers(exprs: &Vec<Box<dyn GwExpression>>,
                                 context: &mut EvaluationContext)
                                 -> Result<Vec<u16>, String> {

    let mut result = Vec::with_capacity(exprs.len());
    for expr in exprs {
        match evaluate_to_usize(expr, context) {
            Ok(usize_val) => {
                result.push(usize_val as u16);
            }
            _ => {
                return Err(String::from("Invalid dimension"));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod dim_tests {
    use std::result::Result;
    use crate::eval::*;    
    use super::*;


    #[test]
    fn it_declares_array_with_dim() -> Result<(), String> {
        let mut program = eval_tests::empty_program();
        let dim = GwDim::new(
                     GwDimDecl::new(
                         "arr".to_string(),
                         vec![Box::new(GwIntegerLiteral::with_value(2))]),
                    None);

        let mut context = eval_tests::empty_context();

        if let Some(_) = context.get_existing_array("arr") {
            return Err("Array already defined!".to_string());
        }

        dim.eval(1, LineExecutionArgument::Empty, &mut context, &mut program);

        if let Some(_) = context.get_existing_array("arr") {
            Ok(())
        } else {
            Err("Array already defined!".to_string())
        } 

    }
}
