use std::{
    borrow::Cow,
    fmt::Display,
    sync::{Arc, Mutex},
};

use aici_abi::{svob::SimpleVob, toktree::TokTrie, MidProcessArg, ProcessResultOffset};
use aici_guidance_ctrl::TokenParser;
use aici_native::bintokens::ByteTokenizerEnv;
use pyo3::{exceptions::PyValueError, prelude::*};

struct EngineInner {
    parser: TokenParser,
    capture_ptr: usize,
}

#[pyclass]
struct Engine {
    inner: Mutex<EngineInner>,
    tok_trie: Arc<TokTrie>,
}

#[pyclass]
struct TokenMask {
    inner: Mutex<SimpleVob>,
    tok_trie: Arc<TokTrie>,
}

fn val_error(e: impl Display) -> PyErr {
    PyValueError::new_err(format!("{e}"))
}

#[pymethods]
impl Engine {
    #[new]
    fn py_new(tokenizer_name: &str, protobuf: &[u8]) -> PyResult<Self> {
        let env = ByteTokenizerEnv::load(tokenizer_name).map_err(val_error)?;
        let tok_trie = env.tok_trie.clone();
        let parser =
            TokenParser::from_guidance_protobuf(Box::new(env), protobuf).map_err(val_error)?;
        Ok(Engine {
            inner: Mutex::new(EngineInner {
                parser,
                capture_ptr: 0,
            }),
            tok_trie: Arc::new(tok_trie),
        })
    }

    fn tokenize(&self, text: &str) -> Vec<u32> {
        let inner = self.inner.lock().unwrap();
        let tokens = inner.parser.token_env.tokenize(text);
        tokens
    }

    fn mid_process(
        &self,
        backtrack: u32,
        tokens: Vec<u32>,
    ) -> PyResult<(Vec<(String, Cow<[u8]>)>, Vec<TokenMask>, String)> {
        let arg = MidProcessArg {
            backtrack,
            tokens,
            fork_group: vec![],
        };
        let mut inner = self.inner.lock().unwrap();
        let r = inner.parser.mid_process(arg);
        let mut token_sets = Vec::new();
        let r2 = ProcessResultOffset {
            branches: r
                .branches
                .iter()
                .map(|b| {
                    b.map_mask(|x| {
                        let idx = token_sets.len();
                        token_sets.push(TokenMask {
                            inner: Mutex::new(x.clone()),
                            tok_trie: self.tok_trie.clone(),
                        });
                        idx
                    })
                })
                .collect(),
        };
        let s = serde_json::to_string(&r2).map_err(val_error)?;
        let captures = inner.parser.parser.captures()[inner.capture_ptr..]
            .iter()
            .map(|(k, v)| (k.clone(), Cow::Owned(v.clone())))
            .collect::<Vec<_>>();
        inner.capture_ptr += captures.len();
        Ok((captures, token_sets, s))
    }
}

#[pymethods]
impl TokenMask {
    fn __repr__(&self) -> PyResult<String> {
        let ts = self.inner.lock().unwrap();
        Ok(self.tok_trie.token_set_dbg(&ts))
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
    m.add_class::<TokenMask>()?;
    Ok(())
}
