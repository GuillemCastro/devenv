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

mod filesystem;
mod devenv;
mod configuration;
mod options;

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate nix;
extern crate semver;
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate packageurl;

use std::fs;
use configuration::Configuration;
use devenv::DevEnv;
use options::Options;
use log::info;
use clap::derive::Clap;

fn main() {
    let options: Options = Options::parse();
    println!("{:?}", options);
    simple_logger::init().unwrap();
    let contents = fs::read_to_string("examples/example.toml").expect("Cannot read the contents of the file");
    let options: Configuration = toml::from_str(contents.as_str()).unwrap();
    println!("{:?}", options);
    
    let devenv = DevEnv::new();

    info!("devenv location {}", devenv.location().unwrap());

    devenv.create();
   
}
