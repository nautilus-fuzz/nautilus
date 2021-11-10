// Nautilus
// Copyright (C) 2020  Daniel Teuchert, Cornelius Aschermann, Sergej Schumilo

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBytes, PyString};

use crate::Context;

#[pyclass]
struct PyContext {
    ctx: Context,
}
impl PyContext {
    fn get_context(&self) -> Context {
        self.ctx.clone()
    }
}

#[pymethods]
impl PyContext {
    #[new]
    fn new() -> Self {
        PyContext {
            ctx: Context::new(),
        }
    }

    fn rule(&mut self, _py: Python, nt: &str, format: &PyAny) -> PyResult<()> {
        if format.is_instance::<PyString>()? {
            let pystr = <&PyString>::extract(&format)?;
            self.ctx.add_rule(nt, pystr.to_string_lossy().as_bytes());
        } else if format.is_instance::<PyBytes>()? {
            let pybytes = <&PyBytes>::extract(&format)?;
            self.ctx.add_rule(nt, pybytes.as_bytes());
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "format argument should be string or bytes",
            ));
        }
        return Ok(());
    }

    fn script(&mut self, nt: &str, nts: Vec<String>, script: PyObject) {
        self.ctx.add_script(nt, nts, script);
    }

    fn regex(&mut self, nt: &str, regex: &str) {
        self.ctx.add_regex(nt, regex);
    }
}

fn main_(py: Python, grammar_path: &str) -> PyResult<Context> {
    let py_ctx = PyCell::new(py, PyContext::new()).unwrap();
    let locals = [("ctx", py_ctx)].into_py_dict(py);
    py.run(
        &std::fs::read_to_string(grammar_path).expect("couldn't read grammar file"),
        None,
        Some(&locals),
    )?;
    return Ok(py_ctx.borrow().get_context());
}

pub fn load_python_grammar(grammar_path: &str) -> Context {
    let gil = Python::acquire_gil();
    let py = gil.python();
    return main_(py, grammar_path)
        .map_err(|e| e.print_and_set_sys_last_vars(py))
        .unwrap();
}
