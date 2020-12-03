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

use std::str::FromStr;
use std::path::PathBuf;
use devenv_common::error::Error;
use std::fs::File;
use std::io;
use io::BufRead;
use nix::{self, mount::MsFlags, };
use std::fmt;

// Some interesting filesystem types for DevEnv
#[derive(Debug, PartialEq)]
pub enum FsType {
    Proc,
    Overlay,
    Tmpfs,
    Sysfs,
    Other(String)
}

impl FromStr for FsType {
    
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proc" => Ok(FsType::Proc),
            "tmpfs" => Ok(FsType::Tmpfs),
            "overlay" => Ok(FsType::Overlay),
            "sysfs" => Ok(FsType::Sysfs),
            &_ => Ok(FsType::Other(s.to_owned()))
        }
    }
}

impl fmt::Display for FsType {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { 
        let fsname = match self {
            FsType::Proc => "proc",
            FsType::Overlay => "overlay",
            FsType::Tmpfs => "tmpfs",
            FsType::Sysfs => "sysfs",
            FsType::Other(s) => s.as_str()
        };
        write!(f, "{}", fsname)
    }

}


#[derive(Debug)]
pub struct MountingPoint {
    pub what: Option<String>,
    pub path: PathBuf,
    pub fstype: Option<FsType>,
    pub options: Option<String>,
    pub flags: Option<MsFlags>,
    pub fatal: Option<bool>,
    pub in_userns: Option<bool>,
    pub use_netns: Option<bool>
}

#[derive(Debug)]
pub struct MTab {
    mounting_points: Vec<MountingPoint>
}

impl MountingPoint {

    pub fn new(what: Option<String>, path: &PathBuf, fstype: Option<FsType>) -> Self {
        return MountingPoint{
            what: what,
            path: path.clone(),
            fstype: fstype,
            options: None,
            flags: None,
            fatal: None,
            in_userns: None,
            use_netns: None
        }
    }

    pub fn new_all(what: Option<String>, path: &PathBuf, fstype: Option<FsType>, options: Option<String>, flags: Option<MsFlags>, fatal: Option<bool>, in_userns: Option<bool>, use_netns: Option<bool>) -> Self {
        return MountingPoint{
            what: what,
            path: path.clone(),
            fstype: fstype,
            options: options,
            flags: flags,
            fatal: fatal,
            in_userns: in_userns,
            use_netns: use_netns
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
                    what: Some(parts[0].to_owned()),
                    path: PathBuf::from(parts[1]),
                    fstype: Some(FsType::from_str(parts[2]).unwrap()),
                    options: Some(parts[3].to_owned()),
                    flags: None,
                    fatal: None,
                    in_userns: None,
                    use_netns: None
                })
            }
        }
        Ok(results)
    }

}

pub fn mount(mounting_point: &MountingPoint) -> Result<(), Error> {
    let source = match &mounting_point.what {
        Some(s) => Some(s.as_str()),
        None => None
    };
    let target = mounting_point.path.to_str().unwrap();
    let tmpfstype = match &mounting_point.fstype {
        Some(fs) => {
            Some(fs.to_string())
        }
        None => None
    };
    let fstype = tmpfstype.as_ref().map(String::as_str);
    let flags = match mounting_point.flags {
        Some(flags) => flags,
        None => MsFlags::MS_RDONLY
    };
    let data = match &mounting_point.options {
        Some(s) => Some(s.as_str()),
        None => None
    };
    match nix::mount::mount::<str, str, str, str>(source, target, fstype, flags, data) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new_error(format!("Could not mount {:?}", mounting_point).as_str(), Box::from(e)))
    }
}