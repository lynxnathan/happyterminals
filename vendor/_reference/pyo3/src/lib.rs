pub mod prelude {
    pub use crate::{PyModule, PyResult, Python};
}

#[derive(Debug, Default)]
pub struct PyErr;

pub type PyResult<T> = Result<T, PyErr>;

#[derive(Debug, Default, Clone, Copy)]
pub struct Python<'py> {
    _marker: core::marker::PhantomData<&'py ()>,
}

#[derive(Debug, Default)]
pub struct PyModule;

impl PyModule {
    pub fn add_function<F>(&self, _name: &str, _function: F) -> PyResult<()> {
        Ok(())
    }
}

