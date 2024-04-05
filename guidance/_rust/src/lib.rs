use std::{fmt::Display, sync::Mutex};

use aici_abi::{self_seq_id, MidProcessArg};
use aici_guidance_ctrl::TokenParser;
use aici_native::bintokens::ByteTokenizerEnv;
use pyo3::{exceptions::PyValueError, prelude::*};

#[pyclass]
struct Engine {
    parser: Mutex<TokenParser>,
}

fn val_error(e: impl Display) -> PyErr {
    PyValueError::new_err(format!("{e}"))
}

#[pymethods]
impl Engine {
    #[new]
    fn py_new(tokenizer_name: &str, protobuf: &[u8]) -> PyResult<Self> {
        let env = ByteTokenizerEnv::load(tokenizer_name).map_err(val_error)?;
        let parser =
            TokenParser::from_guidance_protobuf(Box::new(env), protobuf).map_err(val_error)?;
        Ok(Engine {
            parser: Mutex::new(parser),
        })
    }

    fn mid_process(self_: PyRef<'_, Self>) -> PyResult<String> {
        let arg = MidProcessArg {
            backtrack: 0,
            tokens: vec![],
            fork_group: vec![self_seq_id()],
        };
        let mut p = self_.parser.lock().unwrap();
        let _r = p.mid_process(arg);
        Ok("mid_process".to_string())
    }
}

#[pyfunction]
fn engine_start(parser: &str, grammar: &str, ensure_bos_token: i32) -> PyResult<String> {
    Ok(format!(
        "You passed {} and {} / {}",
        parser, grammar, ensure_bos_token
    ))
}

#[pymodule]
fn guidancerust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    aici_native::setup_log();

    m.add_function(wrap_pyfunction!(engine_start, m)?)?;
    m.add_class::<Engine>()?;
    Ok(())
}
