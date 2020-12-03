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

use std::fs::OpenOptions;
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{fork, ForkResult, getpid, Pid, execvp, execv, chroot};
use nix::sys::wait::waitpid;
use std::ffi::{CString, CStr};
use std::env::{current_exe, set_current_dir, var_os};
use std::fs::{copy, File};
use std::{io::{Read, Write}, path::{PathBuf, Path}};
use devenv_common::error::Error;
use log::{debug, error, warn};
use ipc_channel;
use serde_derive::{Serialize, Deserialize};
use directories::BaseDirs;
use uuid::Uuid;

use devenv_dependencies;
use devenv_common::dependency::Dependency;
use crate::filesystem::Filesystem;

pub struct Container {
    child_pid: Option<Pid>,
    fs: Filesystem,
    ipc: ContainerIPC
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ContainerTask {
    Command {
        name: String,
        params: Vec<String>,
        reuse_pid: bool
    },
    ResolveDependencies(Vec<Dependency>),
    Exit
}

impl Container {

    const INIT_TARGETS: &'static [&'static str] = &["/usr/lib/systemd/systemd", "/lib/systemd/systemd", "/sbin/init"];

    pub fn new(fs: Filesystem) -> Container {
        return Container {
            child_pid: None,
            fs: fs,
            ipc: ContainerIPC::new()
        }
    }

    pub fn create(&mut self) -> Result<(), Error> {
        match self.fs.mount() {
            Ok(_) => {}
            Err(e) => {
                error!("Failed mounting the container's filesystem");
                return Err(e);
            }
        }
        // Copy the binary inside the container
        let bin = current_exe().unwrap();
        match copy(bin, self.fs.root_path().join("usr/bin/devenv")) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to copy DevEnv binary");
                return Err(Error::from(err))
            }
        }
        match unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWCGROUP | CloneFlags::CLONE_NEWIPC) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to unshare");
                return Err(Error::from(err));
            }
        }
        match fork() {
            Ok(ForkResult::Parent {child, ..}) => {
                debug!("(from parent process) Container pid: {}", child);
                self.child_pid = Some(child);
            }
            Ok(ForkResult::Child) => {
                self.container_process().unwrap();
                std::process::exit(0);
            }
            Err(e) => { 
                error!("Fork failed!");
                return Err(Error::from(e)); 
            }
        }
        Ok(())
    }

    pub fn destroy(&self) -> Result<(), Error> {
        self.fs.umount()?;
        self.fs.delete()?;
        Ok(())
    }

    pub fn boot(&self) -> Result<(), Error> {
        for target in Container::INIT_TARGETS {
            self.run_in_container(ContainerTask::Command{name: target.to_string(), params: vec![target.to_string()], reuse_pid: true})?;
        }
        Ok(())
    }

    pub fn wait_for_container(&self) -> Result<(), Error> {
        match waitpid(self.child_pid, None) {
            Ok(status) => { debug!("Child process exited with status {:?}", status) }
            Err(e) => { 
                error!("Child exited with error {}", e);
                return Err(Error::from(e));
            } 
        }
        Ok(())
    }

    fn container_process(&self) -> Result<(), Error> {
        let pid = getpid();
        debug!("(from container process) Container pid: {}", pid);
        match pid.to_string().as_str() {
            "1" => { /* Ok */ }
            _ => {
                // PID for the container must be 1, some applications and services expect to be executed with PID 1 (for example systemd)
                error!("Container is not running with PID 1");
                return Err(Error::new("Container is not running with PID 1"))
            }
        }
        match chroot(&self.root()) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to chroot to {}", self.root().to_str().unwrap_or_default());
                return Err(Error::from(err));
            }
        }
        let base_dirs = BaseDirs::new();
        let new_cwd = match base_dirs {
            None => Path::new("/").to_owned(),
            Some(dirs) => {
                dirs.home_dir().to_owned()
            }
        };
        debug!("Set working directory to {:?}", &new_cwd);
        match set_current_dir(&new_cwd) {
            Ok(_) => (),
            Err(_) => {
                warn!("Could not set working directory to {:?}", new_cwd);
                return Err(Error::new("Could not set working directory"));
            }
        }
        match self.fs.inner_mount() {
            Ok(_) => (),
            Err(e) => {
                error!("Could not mount");
                return Err(e);
            }
        }
        /*match self.setup_boot_id() {
            Ok(_) => (),
            Err (e) => {
                error!("Could not set boot id");
                return Err(Error::from(e))
            }
        }*/
        self.run_tasks();
        Ok(())
    }

    fn run_tasks(&self) {
        debug!("Executing tasks");
        loop {
            match self.ipc.receive() {
                Ok(task) => {
                    self.run_task(task);
                }
                Err(e) => {
                    warn!("Error while receiving new tasks: {}", e)
                }
            }
        }
    }

    fn run_task(&self, task: ContainerTask) {
        debug!("Executing task {:?}", task);
        match task {
            ContainerTask::Command { name, params, reuse_pid } => self.execute_command(name, params, reuse_pid),
            ContainerTask::ResolveDependencies(dependencies) => {
                
                match devenv_dependencies::resolve_dependencies(dependencies) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            ContainerTask::Exit => {

            }
        }
    }

    fn execute_command(&self, filename: String, args: Vec<String>, same_pid: bool) {
        let resolved_filename = match var_os(&filename) {
            None => filename,
            Some(val) => val.to_str().unwrap().to_owned()
        };
        debug!("Executing command {} {:?}", resolved_filename, args);
        let t_filename = CString::new(resolved_filename).unwrap();
        let c_filename = t_filename.as_c_str();
        let t_args: Vec<CString> = args.iter().map(|arg| CString::new(arg.as_bytes()).unwrap()).collect();
        let c_args: Vec<&CStr> = t_args.iter().map(|arg| arg.as_c_str()).collect();
        if same_pid {
            match execvp(c_filename, c_args.as_slice()) {
                    Ok(_) => {}
                    Err(e) => {

                    }
            }
        }
        else {
            match execv(c_filename, c_args.as_slice()) {
                Ok(_) => {}
                Err(e) => {

                }
            }
        }
    }

    pub fn run_in_container(&self, task: ContainerTask) -> Result<(), Error> {
        debug!("Sending task {:?}", task);
        self.ipc.send(task)
    }

    pub fn root(&self) -> PathBuf {
        return self.fs.root_path();
    }

    pub fn location(&self) -> Option<&str> {
        return self.fs.target_path().to_str();
    }

    fn setup_boot_id(&self) -> Result<(), Error> {
        let boot_id = Uuid::new_v4();
        debug!("Boot id: {}", boot_id.to_hyphenated());
        let boot_id_path = Path::new("/proc/sys/kernel/random/boot_id");
        let mut boot_id_file = match OpenOptions::new().write(true).create(true).truncate(true).open(boot_id_path) {
            Ok(file) => file,
            Err(e) => { 
                error!("Could not open boot_id file");
                return Err(Error::from(e))
            }
        };
        //let mut old_id: String = String::new();
        //boot_id_file.read_to_string(&mut old_id)?;
        //debug!("boot id: {}", old_id);
        match boot_id_file.write_all(boot_id.to_hyphenated().to_string().as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::from(e))
            }
        }
        Ok(())
    }

}

pub struct ContainerIPC {
    sender: ipc_channel::ipc::IpcSender<ContainerTask>,
    receiver: ipc_channel::ipc::IpcReceiver<ContainerTask>
}

impl ContainerIPC {

    pub fn new() -> ContainerIPC {
        let (tx, rx) = ipc_channel::ipc::channel().unwrap();
        return ContainerIPC {
            sender: tx,
            receiver: rx
        }
    }

    pub fn send(&self, payload: ContainerTask) -> Result<(), Error> {
        match self.sender.send(payload) {
            Ok(_) => Ok(()),
            Err(err) => {
                Err(Error::new("Error sending task"))
            }
        }
    }

    pub fn receive(&self) -> Result<ContainerTask, Error> {
        match self.receiver.recv() {
            Ok(task) => Ok(task),
            Err(e) => {
                Err(Error::new("Error receiving task"))
            }
        }
    }

}