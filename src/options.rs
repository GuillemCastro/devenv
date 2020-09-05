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

use clap::Clap;

#[derive(Debug)]
#[derive(Clap)]
#[clap(version = "0.1", author = "Guillem Castro <guillemcastro4@gmail.com>")]
pub struct Options {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
    #[clap(long, short, default_value = "./devenv.toml", about = "The configuration file for the DevEnv")]
    pub file: String,
    #[clap(long, short, about = "Activate more verbose output")]
    pub verbose: bool
}

#[derive(Debug)]
#[derive(Clap)]
pub enum SubCommand {
    #[clap(about = "Delete the DevEnv")]
    Delete,
    #[clap(about = "Run a command inside the DevEnv")]
    Run(Run),
    #[clap(about = "Open a shell inside the DevEnv")]
    Shell
}

#[derive(Debug)]
#[derive(Clap)]
pub struct Run {
    pub command: Vec<String>
}