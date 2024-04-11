use super::EditorError;
use crate::util::SynlessBug;
use std::fmt;

pub struct DataStack(Vec<Value>);
pub struct CallStack(Vec<Prog>);

// Executes the last op first
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prog(Vec<Op>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    String(String),
    Quote(Prog),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValueType {
    Int,
    String,
    Quote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    // Data-stack manipulation:
    Swap,
    Apply,
    Pop,
    Literal(Value),

    // Keymap:
    Block,
    ExitMenu,
}

impl Value {
    fn get_type(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::String(_) => ValueType::String,
            Value::Quote(_) => ValueType::Quote,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ValueType::{Int, Quote, String};

        match self {
            Int => write!(f, "Int"),
            String => write!(f, "String"),
            Quote => write!(f, "Quote"),
        }
    }
}

impl Prog {
    /// Construct a program in which the given ops will be executed in order,
    /// starting from index 0 of the slice.
    pub fn new(forward_ops: Vec<Op>) -> Self {
        let mut ops = forward_ops;
        ops.reverse();
        Prog(ops)
    }

    /// Insert `op` as the _first_ operator to be executed in this program.
    pub fn insert_first(&mut self, op: Op) {
        self.0.push(op);
    }

    /// Insert `op` as the _last_ operator to be executed in this program.
    pub fn insert_last(&mut self, op: Op) {
        self.0.insert(0, op);
    }

    /// Produce a literal value containing this program.
    pub fn quote(self) -> Value {
        Value::Quote(self)
    }

    /// Pop the next op from the program.
    fn pop(&mut self) -> Option<Op> {
        self.0.pop()
    }
}

impl From<Op> for Prog {
    /// Construct a program containing one op.
    fn from(op: Op) -> Prog {
        Prog(vec![op])
    }
}

impl CallStack {
    /// Construct a new empty callstack.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Push a program onto the call stack.
    pub fn push(&mut self, prog: Prog) {
        self.0.push(prog)
    }

    /// Return the next op to execute, removing it from the call stack. Or
    /// return None if the call stack is empty.
    pub fn pop(&mut self) -> Option<Op> {
        loop {
            let prog = self.0.last_mut()?;
            let op = prog.pop();
            if op.is_some() {
                if prog.0.is_empty() {
                    // Do a sort of tail-call optimization, immediately removing
                    // the empty program from the stack.
                    self.0.pop();
                }
                return op; // Success!
            }
            // Remove empty program and try again
            self.0.pop();
        }
    }
}

impl DataStack {
    /// Construct a new empty data stack.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Swap the order of the two top-most values on the data stack. Return an
    /// error if there are less than 2 values on the data stack.
    pub fn swap(&mut self) -> Result<(), EditorError> {
        let first = self.pop()?;
        let maybe_second = self.pop();
        self.push(first);
        self.push(maybe_second?);
        Ok(())
    }

    /// Push this value onto the data stack.
    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    /// Pop a value from the data stack, returning it. If the stack is empty,
    /// return an error.
    pub fn pop(&mut self) -> Result<Value, EditorError> {
        self.0.pop().ok_or(EditorError::EmptyDataStack)
    }

    /// If there is a `Value::Int` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_int(&mut self) -> Result<i32, EditorError> {
        match self.pop()? {
            Value::Int(num) => Ok(num),
            other => {
                let err = type_mismatch_error(&other, ValueType::Int);
                self.push(other);
                Err(err)
            }
        }
    }

    /// If there is a `Value::String` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_string(&mut self) -> Result<String, EditorError> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            other => {
                let err = type_mismatch_error(&other, ValueType::String);
                self.push(other);
                Err(err)
            }
        }
    }

    /// If there is a `Value::Quote` on top of the stack, pop it and return it.
    /// Otherwise return an error.
    pub fn pop_quote(&mut self) -> Result<Prog, EditorError> {
        match self.pop()? {
            Value::Quote(prog) => Ok(prog),
            other => {
                let err = type_mismatch_error(&other, ValueType::Quote);
                self.push(other);
                Err(err)
            }
        }
    }
}

fn type_mismatch_error(actual: &Value, expected: ValueType) -> EditorError {
    EditorError::TypeMismatch {
        actual: actual.get_type().to_string(),
        expected: expected.to_string(),
    }
}
