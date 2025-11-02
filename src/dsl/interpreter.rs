/// Runtime interpreter for structured YAML behavior specifications

use crate::dsl::errors::{DslError, DslResult};
use crate::dsl::structured_format::*;

pub struct Interpreter {
    pub behavior_spec: StructuredBehaviorSpec,
}

impl Interpreter {
    pub fn new(behavior_spec: StructuredBehaviorSpec) -> Self {
        Interpreter { behavior_spec }
    }

    // TODO: Implement interpreter
    // This will execute state actions at runtime
}
