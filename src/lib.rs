#![crate_name = "vg_errortools"]
//! # Tooling for better human readable errors
//!
#![warn(missing_docs)]
#![warn(unused_qualifications)]
#![deny(deprecated)]

use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
#[cfg(feature = "tokio")]
use std::future::Future;
use std::path::{Path, PathBuf};

/// # A wrapper for io::Error which also contains the file path it failed on
/// This error comprises a `std::io::Error` as source and a `Pathbuf` containing the file path the operation failed on.
/// Consider using this together with [`fat_io_wrap_std`] for std::io functions.
/// With the feature 'tokio' there's also: `fat_io_wrap_tokio` for tokio-async based functions.
#[derive(Debug)]
pub struct FatIOError {
    source: std::io::Error,
    file: PathBuf,
}

impl FatIOError {
    /// manually create a FatIOError from an std error when the file is still known
    pub fn from_std_io_err(e: std::io::Error, file: PathBuf) -> Self {
        FatIOError { source: e, file }
    }
}

impl Display for FatIOError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operating on file '{}' failed with error {}",
            self.file.to_string_lossy(),
            self.source
        )?;
        Ok(())
    }
}

impl Error for FatIOError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// # Wrapper for std::io functions
/// This runs any std::io function which only takes a single argument `impl AsRef<Path>` and wraps the filename argument in a [`FatIOError`] if one occurs.
/// Since this operation involves a Pathbuf-Deepcopy it's not free, so be careful in high frequency contexts.
/// # Examples
/// ```rust, no_run
/// use std::fs::File;
/// use errortools::fat_io_wrap_std;
/// let file_result = fat_io_wrap_std("my_file.txt", &File::open);
/// ```
///
/// ```rust, no_run
/// use std::fs::read_to_string;
/// use errortools::fat_io_wrap_std;
/// let to_string_result = fat_io_wrap_std("my_file.txt", &read_to_string);
/// ```
///
pub fn fat_io_wrap_std<T, P: AsRef<Path>>(
    path: P,
    f: &dyn Fn(P) -> std::io::Result<T>,
) -> Result<T, FatIOError> {
    let path_buf = path.as_ref().to_path_buf();
    let result = f(path);
    result.map_err(|e| FatIOError {
        source: e,
        file: path_buf,
    })
}

/// # Wrapper for tokio::fs functions
/// This runs any tokio::fs function which only takes a single argument `impl AsRef<Path>` and wraps the filename argument in a [`FatIOError`] if one occurs.
/// Since this operation involves a Pathbuf-Deepcopy it's not free, so be careful in high frequency contexts.
/// # Examples
/// ```rust, no_run
/// use errortools::{fat_io_wrap_tokio};
/// async fn some_fn() -> Result<tokio::fs::File, errortools::FatIOError> {
///   fat_io_wrap_tokio("my_file.txt", tokio::fs::File::open).await
/// }
/// ```
///
/// ```rust, no_run
/// use errortools::{fat_io_wrap_tokio};
/// async fn some_fn() -> Result<String, errortools::FatIOError> {
///   fat_io_wrap_tokio("my_file.txt", tokio::fs::read_to_string).await
/// }
/// ```
///
#[cfg(feature = "tokio")]
pub async fn fat_io_wrap_tokio<T, P: AsRef<Path>, F: Future<Output = std::io::Result<T>>>(
    path: P,
    f: fn(P) -> F,
) -> Result<T, FatIOError> {
    let path_buf = path.as_ref().to_path_buf();
    let result = f(path).await;
    result.map_err(|e| FatIOError {
        source: e,
        file: path_buf,
    })
}

/// # An error wrapper for usage in the main functions printing better human readable errors from e.g. `thiserror` crate.
/// Examples:
/// ```rust, no_run
/// use errortools::MainError;
/// use thiserror::Error;
/// #[derive(Error, Debug)]
/// pub enum SubError {
///   #[error("This text will be printed when ?-Operator is used in main now, thanks 'MainError'!")]
///   MyVariant
/// }
/// pub fn main() -> Result<(), MainError> {
///     let my_function_result = Err(SubError::MyVariant)?;
///     Ok(())
/// }
/// ```
pub struct MainError(Box<dyn Error>);

impl<E: Into<Box<dyn Error>>> From<E> for MainError {
    fn from(e: E) -> Self {
        MainError(e.into())
    }
}

impl Debug for MainError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)?;
        let mut source = self.0.source();
        while let Some(error) = source {
            write!(f, "\ncaused by: {}", error)?;
            source = error.source();
        }
        Ok(())
    }
}
