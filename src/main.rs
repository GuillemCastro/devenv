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
mod container;
mod lib;

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
use crate::devenv::DevEnv;
use options::Options;
use clap::derive::Clap;

fn main() {
    let options: Options = Options::parse();
    
    match options.verbose {
        true => simple_logger::init(),
        false => simple_logger::init_with_level(log::Level::Warn)
    }.expect("Couldn't configure the logging level");

    debug!("{:?}", options);

    let contents = fs::read_to_string(options.file).expect("Cannot read the contents of the file");
    let config: Configuration = toml::from_str(contents.as_str()).unwrap();
    
    debug!("{:?}", config);
    
    let mut devenv = DevEnv::from(config);
    info!("devenv location: {}", devenv.location().unwrap());
    devenv.create().unwrap();

    match options.subcmd {
        options::SubCommand::Delete => {
            devenv.destroy().unwrap()
        }
        options::SubCommand::Run(run) => {
            let args = run.command;
            let command = args[0].clone();
            devenv.run(command, args).unwrap()
        }
        options::SubCommand::Shell => {
            devenv.open_shell().unwrap()
        }
    }

    devenv.wait_for_container().unwrap();
}
