mod filesystem;
mod error;
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
