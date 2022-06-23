use anyhow::{anyhow, Result};

pub trait ErrorGenerico<T> {
    fn error_generico(self) -> Result<T>;
}

impl<T, E> ErrorGenerico<T> for std::result::Result<T, E> {
    fn error_generico(self) -> Result<T> {
        self.map_err(|_| anyhow!("Algo salio mal :("))
    }
}

impl<T> ErrorGenerico<T> for Option<T> {
    fn error_generico(self) -> Result<T> {
        self.ok_or_else(|| anyhow!("Algo salio mal :("))
    }
}
