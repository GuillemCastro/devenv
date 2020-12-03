/**
 * The MIT License
 * Copyright (c) 2020 Guillem Castro
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

use std::error;
use std::fmt;
use std::io;
use std::convert::From;
use nix;

/// The error type for DevEnv. Most errors originate in the OS. But sometimes 
/// errors will be crafted with other causes. For more info about the error, you
/// can check the `ErrorKind`.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    msg: String
}

/// Some relevant types of errors. Do not exhaustively match against them, as they
/// might grow over time.
#[derive(Debug)]
pub enum ErrorKind {
    UnixError(nix::Error),
    IOError(io::Error),
    Custom,
    Other(Box<dyn error::Error + Send + Sync>)
}

impl Error {

    pub fn new(msg: &str) -> Self {
        return Error {
            kind: ErrorKind::Custom,
            msg: msg.to_owned()
        }
    }

    pub fn new_error(msg: &str, err: Box<dyn error::Error + Send + Sync>) -> Self {
        return Error {
            kind: ErrorKind::Other(err),
            msg: msg.to_owned()
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        self.msg.as_str()
    }

}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
    }

}

impl fmt::Display for ErrorKind {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ErrorKind::UnixError(ref err) => write!(f, "UnixError({})", err),
            ErrorKind::IOError(ref err) => write!(f, "IOError({})", err),
            ErrorKind::Other(ref err) => err.fmt(f),
            ErrorKind::Custom => write!(f, "")
        }
    }
}

impl error::Error for Error {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.kind {
            ErrorKind::UnixError(ref err) => Some(err),
            ErrorKind::IOError(ref err) => Some(err),
            ErrorKind::Other(ref err) => err.source(),
            _ => None
        }
    }

}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        let msg = error.to_string();
        return Error {
            kind: ErrorKind::IOError(error),
            msg: msg
        }
    }
}

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        return Error {
            kind: ErrorKind::UnixError(error),
            msg: error.to_string()
        }
    }
}

impl From<Box<dyn error::Error + Send + Sync>> for Error {

    fn from(e: Box<dyn error::Error + Send + Sync>) -> Self {
        let msg = e.to_string();
        return Error {
            kind: ErrorKind::Other(e),
            msg: msg
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(k: ErrorKind) -> Self {
        return Error {
            kind: k,
            msg: "".to_owned()
        }
    }
}