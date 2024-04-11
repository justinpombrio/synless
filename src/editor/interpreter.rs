use super::runtime::Runtime;
use super::stack::{CallStack, DataStack, Op, Prog};
use super::EditorError;

#[derive(Debug, Default)]
pub struct Interpreter {
    call_stack: CallStack,
    data_stack: DataStack,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter::default()
    }

    /// Execute the program. Returns `Ok(true)` if it halted from a `Block` op,
    /// and `Ok(false)` if it halted from hitting the end of the program.
    pub fn execute(&mut self, r: &mut Runtime, prog: Prog) -> Result<bool, EditorError> {
        self.call_stack.push(prog);
        while let Some(op) = self.call_stack.pop() {
            if op == Op::Block {
                return Ok(true);
            } else {
                self.call(r, op)?;
            }
        }
        Ok(false)
    }

    fn call(&mut self, r: &mut Runtime, op: Op) -> Result<(), EditorError> {
        todo!()
    }
}
