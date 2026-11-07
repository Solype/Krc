use std::collections::BTreeMap;

use crate::parser::postparser::structs::Program;

use super::convert_error::ConvertError;
use super::ir;

pub struct Converter<'a> {
    pub(super) program: &'a Program,
    /// Mapping the mangled function name with its index
    pub(super) functions: BTreeMap<String, usize>,
    /// The IR being built: its strings and floats are filled while the functions are converted
    pub(super) ir: ir::structs::Program,
    /// The program is one file: it must have its `main`
    pub(super) needs_main: bool,
}

impl<'a> Converter<'a> {
    pub fn new(program: &'a Program, needs_main: bool) -> Self {
        Self {
            program,
            needs_main,
            functions: BTreeMap::new(),
            ir: ir::structs::Program::default(),
        }
    }

    pub fn convert(&mut self) -> Result<ir::structs::Program, ConvertError> {
        // Get the global variables
        // Get the structures
        self.get_all_functions()?;
        self.check_main()?;
        let program = self.program;
        for func in &program.functions {
            let fnc = self.convert_func(func)?;
            self.ir.funcs.push(fnc);
        }
        // for func in self.program.functions.iter() {}
        Ok(std::mem::take(&mut self.ir))
    }
}
