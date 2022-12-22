//
// Macro for "converting"
// from Result::Err to InstructionResult::EvaluateToError
//
#[macro_export]
macro_rules! check_result {
    ( $x:expr ) => {
        if let Err(err) = $x {
            return InstructionResult::EvaluateToError(err.into());
        }
    }
}
