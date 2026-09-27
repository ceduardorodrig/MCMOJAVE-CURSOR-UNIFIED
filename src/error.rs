// Copyright (C) 2026 Carlos Eduardo Rodrigues
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Process '{command}' failed (exit {code:?}): {stderr}")]
    #[allow(dead_code)]
    ProcessFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("{0}")]
    Custom(String),
}
