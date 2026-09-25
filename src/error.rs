// Copyright (C) 2026 Carlos Eduardo Rodrigues
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    ProcessFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[allow(dead_code)]
    Custom(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Zip(err) => write!(f, "ZIP error: {err}"),
            Self::ProcessFailed {
                command,
                code,
                stderr,
            } => {
                write!(
                    f,
                    "Process '{command}' failed with exit code {:?}: {stderr}",
                    code
                )
            }
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::Zip(err)
    }
}
