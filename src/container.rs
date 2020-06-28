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

use nix::sched::{unshare, CloneFlags};
use nix::unistd::{fork, ForkResult, getpid, Pid, execvp, chroot};
use nix::sys::wait::waitpid;
use std::ffi::CString;
use std::env::current_exe;
use std::fs::copy;
use std::path::PathBuf;

use crate::filesystem::Filesystem;

pub struct Container {
    child_pid: Option<Pid>,
    fs: Filesystem
}

impl Container {

    pub fn new(fs: Filesystem) -> Container {
        return Container {
            child_pid: None,
            fs: fs
        }
    }

    pub fn create(&mut self) -> Result<(), ()> {
        self.fs.mount().expect("Failed mounting the container's root filesystem");
        // Copy the binary inside the container
        let bin = current_exe().unwrap();
        copy(bin, self.fs.root_path().join("usr/bin/devenv")).expect("Failed when copying devenv binary to the container");
        unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWCGROUP | CloneFlags::CLONE_NEWIPC)
            .expect("unshare failed");
        match fork() {
            Ok(ForkResult::Parent {child, ..}) => {
                println!("(from parent) Child pid: {}", child);
                self.child_pid = Some(child);
                match waitpid(self.child_pid, None) {
                    Ok(status) => { println!("Child process exited with status {:?}", status) }
                    Err(e) => { panic!("Child exited with error {}", e) }
                }
            }
            Ok(ForkResult::Child) => {
                println!("(from child) Child pid: {}", getpid());
                chroot(&self.root()).expect("chroot failed");
                let filename = CString::new("/bin/bash").unwrap();
                let empty = CString::new("").unwrap();
                let args = vec![empty.as_c_str()];
                execvp(filename.as_c_str(), &args.as_slice()).expect("Execution failed");
            }
            Err(_) => { panic!("Fork failed") }
        }
        Ok(())
    }

    pub fn root(&self) -> PathBuf {
        return self.fs.root_path();
    }

    pub fn location(&self) -> Option<&str> {
        return self.fs.target_path().to_str();
    }

}