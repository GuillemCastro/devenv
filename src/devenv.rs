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

use crate::filesystem::Filesystem;
use crate::configuration::Configuration;
use crate::container::Container;

use std::fs;
use std::env;
use std::path::PathBuf;

pub struct DevEnv {
    container: Container
}

impl DevEnv {

    const DEFAULT_IMAGE: &'static str = "/";

    const DEFAULT_TARGET: &'static str = ".devenv";

    pub fn new() -> DevEnv {
        let target = env::current_dir().unwrap().join(DevEnv::DEFAULT_TARGET);
        let fs = Filesystem::new(&DevEnv::DEFAULT_IMAGE, &target);
        return DevEnv {
            container: Container::new(fs)
        }
    }

    pub fn create(&mut self) {
        self.container.create().expect("Could not create devenv");
    }

    pub fn location(&self) -> Option<&str> {
        return self.container.location();
    }

}

impl From<Configuration> for DevEnv {

    fn from(config: Configuration) -> DevEnv {
        let image = match &config.image {
            Some(i) => { &i.path }
            None => { DevEnv::DEFAULT_IMAGE }
        };
        let destination = match &config.dest {
            Some(dest) => { PathBuf::from(dest) }
            None => { env::current_dir().unwrap().join(DevEnv::DEFAULT_TARGET) }
        };
        let fs = Filesystem::new(&image, &destination);
        return DevEnv {
            container: Container::new(fs)
        }
    }

}