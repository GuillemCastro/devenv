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

use std::fs::File;
use std::path::{PathBuf};
use std::io::BufRead;
use std::io;

use std::convert::From;

use nix;

#[derive(Debug)]
pub enum Error {
    MountError(String),
    UnixError(String, Option<nix::Error>),
    IPCError(String),
    IOError(String, Option<io::Error>)
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::MountError(ref msg) => write!(f, "MountError: {}", msg),
            Error::UnixError(ref msg, None) => write!(f, "UnixError: {}", msg),
            Error::UnixError(ref msg, Some(ref err)) => write!(f, "UnixError: {} {}", msg, err),
            Error::IPCError(ref msg) => write!(f, "IPCError: {}", msg),
            Error::IOError(ref msg, None) => write!(f, "IOError: {}", msg),
            Error::IOError(ref msg, Some(ref err)) => write!(f, "IOError: {} {}", msg, err)
        }
    }

}

impl error::Error for Error {

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::UnixError(.., Some(ref err)) => Some(err),
            Error::IOError(.., Some(ref err)) => Some(err),
            _ => None
        }
    }

}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError(error.to_string(), Some(error))
    }
}

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        Error::UnixError(error.to_string(), Some(error))
    }
}


pub struct MountingPoint {
    path: PathBuf,
    fstype: String,
}

pub struct MTab {
    mounting_points: Vec<MountingPoint>
}

impl MountingPoint {

    pub fn new(path: &PathBuf, fstype: &str) -> Self {
        return MountingPoint{
            path: path.clone(),
            fstype: fstype.to_owned()
        }
    }

}

impl MTab {

    pub fn new() -> Self {
        return MTab {
            mounting_points: MTab::get_mounting_points().unwrap()
        }
    }

    pub fn contains(&self, mounting_point: MountingPoint) -> bool {
        let filtered: Vec<&MountingPoint> = self.mounting_points.iter().
            filter(|mts| 
                mts.path == mounting_point.path && mts.fstype == mounting_point.fstype)
            .collect();
        filtered.len() > 0
    }

    pub fn get_mounting_points() -> Result<Vec<MountingPoint>, Error> {
        let mut results: Vec<MountingPoint> = vec![];
        let mtab = File::open("/etc/mtab")?;
        let reader = io::BufReader::new(mtab);
        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if !parts.is_empty() {
                results.push(MountingPoint {
                    path: PathBuf::from(parts[1]),
                    fstype: parts[2].to_owned()
                })
            }
        }
        Ok(results)
    }

}