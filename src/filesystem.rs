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

use libmount::{Overlay, Tmpfs};
use std::path::{Path, PathBuf};
use std::fs;
use log::{warn};
use crate::lib::Error;

use semver::Version;
use nix::sys::utsname::uname;
use nix::mount::{mount, MsFlags, umount};
use devenv::{MountingPoint, MTab};

pub struct Filesystem {
    imagepath: PathBuf,
    targetpath: PathBuf
}

impl Filesystem {

    const MERGE_DIR: &'static str = "merge";

    const UPPER_DIR: &'static str = "upper";

    const WORK_DIR: &'static str = "workdir";

    pub fn new(image: &impl AsRef<Path>, target: &impl AsRef<Path>) -> Filesystem {
        // Overlayfs was introduced in kernel version 3.18
        match Filesystem::is_kernel_version_compatible("3.18.0") {
            Ok(true) => {} // The Kernel version is compatible
            _ => { // It might not be compatible, but anyways we can try so just log a warning
                warn!("Your kernel version might not be compatible with devenv. Use version 3.18 or greater for better compatibility");
            }
        }
        return Filesystem {
            imagepath: image.as_ref().to_path_buf(),
            targetpath:  target.as_ref().to_path_buf(),
        };
    }

    /// Mount an overlayfs that will be used as the filesystem for the container.
    /// 
    /// Overlayfs works by combining several layers of read-only directories (lowerdirs), with a read/write 
    /// directory on top (upperdir). The writes to the resulting filesystem will be saved in the upperdir.
    /// 
    /// This is how the Overlayfs directories look in DevEnv 
    ///     lowerdirs = the container image
    ///     upperdir = .devenv/upper
    ///     workdir = .devenv/work
    ///     target = .devenv/merge
    /// As a tree,
    ///     .devenv/
    ///         upperdir/
    ///         workdir/
    ///         merge/
    /// 
    /// It is possible to use the current rootfs as the container's image, but as Linux does not allow
    /// to have circular references inside the same filesystem we must put the Overlayfs inside another
    /// filesystem. In this case, a tmpfs. The downside of this election is that the contents of a tmpfs
    /// are stored in memory, and changes are lost when rebooting.
    /// 
    pub fn mount(&self) -> Result<(), Error> {
        let mtab = MTab::new();
        if !self.targetpath.exists() {
            fs::create_dir(&self.targetpath).unwrap();
        }
        // If the image path contains the target path, we need to mount the merge/target
        // directory inside a Tmpfs. Otherwise it will fail due to cyclic references.
        if self.targetpath.ancestors().any(|x| x == self.imagepath) { 
            warn!("The image path contains the devenv path. All non-persisted changes will be lost at reboot.");
            if !mtab.contains(MountingPoint::new(&self.targetpath, "tmpfs")) {
                let targetfs = Tmpfs::new(&self.targetpath);
                match targetfs.mount() {
                    Ok(_) => {}
                    Err(e) => { return Err(Error::MountError("Could not mount tmpfs".to_owned())) }
                }
            }
        }
        // Before mounting, create the Overlay directories
        if !self.targetpath.join(Filesystem::MERGE_DIR).exists() {
            fs::create_dir(self.targetpath.join(Filesystem::MERGE_DIR)).unwrap();
        }
        if !self.targetpath.join(Filesystem::UPPER_DIR).exists() {
            fs::create_dir(self.targetpath.join(Filesystem::UPPER_DIR)).unwrap();
        }
        if !self.targetpath.join(Filesystem::WORK_DIR).exists() {
            fs::create_dir(self.targetpath.join(Filesystem::WORK_DIR)).unwrap();
        }
        if !mtab.contains(MountingPoint::new(&self.targetpath.join(Filesystem::MERGE_DIR), "overlay")) {
            let overlayfs = Overlay::writable(
                vec![&self.imagepath].iter().map(|x| x.as_path()), 
                self.targetpath.join(Filesystem::UPPER_DIR), 
                self.targetpath.join(Filesystem::WORK_DIR), 
                self.targetpath.join(Filesystem::MERGE_DIR),
            );
            match overlayfs.mount() {
                Ok(_) => {}
                Err(e) => { return Err(Error::MountError("Could not mount overlayfs".to_owned())) }
            }
        }
        Ok(())
    }

    /// Mounts the procfs of the container
    /// 
    /// Must be run AFTER chrooting, otherwise bad things might happen.
    pub fn mount_procfs(&self) -> Result<(), Error> {
        // mount -t proc proc /proc
        match mount::<str, str, str, str>(None, "/proc", Some("proc"), MsFlags::MS_RDONLY, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::UnixError("Could not mount procfs".to_owned(), Some(e)))
        }
    }

    pub fn umount(&self) -> Result<(), Error> {
        umount(&self.targetpath.join("merge").join("proc"))?;
        umount(&self.targetpath.join("merge"))?;
        umount(&self.targetpath)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), Error> {
        let mtab = MTab::new();
        // If the image is set to be the rootfs, we don't want to accidentaly delete it
        while mtab.contains(MountingPoint::new(&self.targetpath.join(Filesystem::MERGE_DIR), "overlay")) {
            self.umount()?;
        }
        fs::remove_dir_all(self.targetpath.join(Filesystem::MERGE_DIR))?;
        fs::remove_dir_all(self.targetpath.join(Filesystem::UPPER_DIR))?;
        fs::remove_dir_all(self.targetpath.join(Filesystem::WORK_DIR))?;
        fs::remove_dir_all(&self.targetpath)?;
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

    /// Check if the current kernel version is greater than the required version
    fn is_kernel_version_compatible(required_kernel_version: &str) -> Result<bool, Error> {
        let sysinfo = uname();
        let sysversion = sysinfo.release();
        let current_version = Version::parse(sysversion).unwrap();
        let required_version = Version::parse(required_kernel_version).unwrap();
        return Ok(current_version.ge(&required_version));
    }

}