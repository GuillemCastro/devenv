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

use packageurl::PackageUrl;
use std::str::FromStr;
use std::{fmt::Debug};
use serde_derive::{Deserialize, Serialize};
use crate::error::Error;

#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
pub struct Dependency {
    pub purl: Option<String>,
    pub provider: Option<String>,
    pub package: Option<String>,
    pub version: Option<String>
}

impl Dependency {

    pub fn new(provider: &str, package: &str, version: &str) -> Self {
        return Dependency {
            purl: None,
            provider: Some(provider.to_owned()),
            package: Some(package.to_owned()),
            version: Some(version.to_owned())
        }
    }
    
    pub fn provider(&self) -> Result<String, Error> {
        match &self.provider {
            Some(s) => { Ok(s.clone()) }
            None => {
                match PackageUrl::from_str(self.purl.as_ref().unwrap().as_str()) {
                    Ok(pkg) => { Ok(pkg.ty.into_owned()) }
                    Err(e) => { Err(Error::new("Could not parse provider")) }
                }
            }
        }
    }

    pub fn package(&self) -> Result<String, Error> {
        match &self.package {
            Some(s) => { Ok(s.clone()) }
            None => {
                match PackageUrl::from_str(self.purl.as_ref().unwrap().as_str()) {
                    Ok(pkg) => { Ok(pkg.name.into_owned()) }
                    Err(e) => { Err(Error::new("Could not parse package")) }
                }
            }
        }
    }

    pub fn version(&self) -> Result<String, Error> {
        match &self.version {
            Some(s) => { Ok(s.clone()) }
            None => {
                match PackageUrl::from_str(self.purl.as_ref().unwrap().as_str()) {
                    Ok(pkg) => { 
                        match pkg.version {
                            Some(s) => { Ok(s.into_owned()) }
                            None => Ok("".to_string())
                        }
                    }
                    Err(e) => { Err(Error::new("Could not parse version")) }
                }
            }
        }
    }

}
