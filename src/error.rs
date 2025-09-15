use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Error loading stdin: {0}")]
    Io(#[from] io::Error),

    #[error("Required info missing from input: {0:?}")]
    MissingInfo(String),

    #[error("Provided information invalid: {0:?}")]
    InvalidInfo(String),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error constructing database paths")]
    Path,

    #[error("Error managing database file: {0}")]
    Io(#[from] io::Error),

    #[error("Rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Error, Debug)]
pub enum GithubError {
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Missing field {0} in api response")]
    MissingField(String),

    #[error("API response field {0} contained invalid or unexpected data")]
    InvalidField(String),

    #[error("Timed out waiting for authentication. Token only valif for {0}s")]
    Timeout(i32),

    #[error("Keyring failed to store token: {0}")]
    KeyingError(#[from] keyring::Error),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    GithubError(#[from] GithubError),

    #[error("{0}")]
    GitError(#[from] GitError),

    #[error("{0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("{0}")]
    PromptError(#[from] inquire::error::InquireError),
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs;
//
//     #[test]
//     fn test_error() {
//         let io_error = fs::read_to_string("fake/path.txt");
//         assert!(io_error.is_err());
//         if let Err(err) = io_error {
//             assert_eq!(format!("{}", err), format!("{}", GitRequestError::Io(err)))
//         }
//     }
// }
