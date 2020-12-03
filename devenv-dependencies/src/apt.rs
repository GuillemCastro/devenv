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

use crate::provider::{DependencyInfo, DependencyProvider};
use devenv_common::dependency::Dependency;
use devenv_common::error::Error;
use apt_pkg_native::{simple, Cache};
use log::debug;

pub struct APTProvider {

    name: &'static str,

}

impl DependencyProvider for APTProvider {

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn search(&self, dependency: &Dependency) -> Result<Vec<Dependency>, Error> {
        let mut cache = Cache::get_singleton();
        let pkg_name = match dependency.package() {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::from(e));
            }
        };
        let items = match cache.find_by_name(pkg_name.as_str()).next() {
            Some(pkg) => {
                let deps: Vec<Dependency> = pkg.versions().map(
                    |f| Dependency::new(
                        self.name, pkg.name().as_str(), f.version().as_str()
                    )
                ).collect();
                deps
            }
            None => {
                return Err(Error::new("No packages available"))
            }
        };
        Ok(items)
    }

    fn info(&self, dependency: &Dependency) -> Result<& dyn DependencyInfo, Error> {
        todo!()
    }

    fn install(&self, dependency: &Dependency) -> Result<(), Error> {
        todo!()
    }
}

impl APTProvider {

    pub fn new() -> Self {
        return APTProvider {
            name: "apt"
        }
    }

}