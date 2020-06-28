use libmount::{Overlay, Tmpfs};
use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;
use log::{warn};

use semver::Version;
use nix::sys::utsname::uname;

pub struct Filesystem {
    imagepath: PathBuf,
    targetpath: PathBuf
    //overlayfs: Overlay,
    //targetfs: Option<Tmpfs>
}

impl Filesystem {

    pub fn new(image: &impl AsRef<Path>, target: &impl AsRef<Path>) -> Filesystem {
        match Filesystem::is_kernel_version_compatible("3.18.0") {
            Ok(true) => {} // The Kernel version is compatible
            _ => { // It might not be compatible
                warn!("Your kernel version might not be compatible with devenv. Use version 3.18 or greater for better compatibility");
            }
        }
        return Filesystem {
            imagepath: image.as_ref().to_path_buf(),
            targetpath:  target.as_ref().to_path_buf(),
        };
    }

    pub fn mount(&self) -> Result<(), Box<dyn Error>> {
        if !self.targetpath.exists() {
            fs::create_dir(&self.targetpath)?;
        }
        // If the image path contains the target path, we need to mount the merge/target
        // directory inside a Tmpfs. Otherwise it will fail due to cyclic references.
        if self.targetpath.ancestors().any(|x| x == self.imagepath) { 
            warn!("The image path contains the devenv path. All non-persisted changes will be lost at reboot.");
            let targetfs = Tmpfs::new(&self.targetpath);
            match targetfs.mount() {
                Ok(_) => {}
                Err(e) => { return Err(Box::from(e)) }
            }
        }
        // Before mounting, create the Overlay directories
        if !self.targetpath.join("merge").exists() {
            fs::create_dir(self.targetpath.join("merge"))?;
        }
        if !self.targetpath.join("upper").exists() {
            fs::create_dir(self.targetpath.join("upper"))?;
        }
        if !self.targetpath.join("workdir").exists() {
            fs::create_dir(self.targetpath.join("workdir"))?;
        }
        let overlayfs = Overlay::writable(
            vec![&self.imagepath].iter().map(|x| x.as_path()), 
            self.targetpath.join("upper"), 
            self.targetpath.join("workdir"), 
            self.targetpath.join("merge"),
        );
        match overlayfs.mount() {
            Ok(_) => {}
            Err(e) => { return Err(Box::from(e)) }
        }
        Ok(())
    }

    pub fn root_path(&self) -> PathBuf {
        return self.targetpath.join("merge");
    }

    pub fn image_path(&self) -> &PathBuf {
        return &self.imagepath;
    }

    pub fn target_path(&self) -> &PathBuf {
        return &self.targetpath;
    }

    fn is_kernel_version_compatible(required_kernel_version: &str) -> Result<bool, Box<dyn Error>> {
        let sysinfo = uname();
        let sysversion = sysinfo.release();
        let current_version = Version::parse(sysversion)?;
        let required_version = Version::parse(required_kernel_version)?;
        return Ok(required_version.ge(&current_version));
    }

}