use crate::filesystem::Filesystem;
use crate::configuration::Configuration;

use std::fs;
use std::env;
use std::path::PathBuf;

pub struct DevEnv {
    fs: Filesystem
}

impl DevEnv {

    const DEFAULT_IMAGE: &'static str = "/";

    const DEFAULT_TARGET: &'static str = ".devenv";

    pub fn new() -> DevEnv {
        let target = env::current_dir().unwrap().join(DevEnv::DEFAULT_TARGET);
        return DevEnv {
            fs: Filesystem::new(&DevEnv::DEFAULT_IMAGE, &target)
        }
    }

    pub fn create(&self) {
        match self.fs.mount() {
            Ok(_) => {}
            Err(e) => panic!("Cannot create devenv: {}", e)
        }
        // Copy the binary inside the container
        let bin = env::current_exe().unwrap();
        fs::copy(bin, self.fs.root_path().join("usr/bin/devenv")).unwrap();
    }

    pub fn location(&self) -> Option<&str> {
        return self.fs.target_path().to_str();
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
        return DevEnv {
            fs: Filesystem::new(&image, &destination)
        }
    }

}