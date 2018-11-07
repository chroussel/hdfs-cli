extern crate rand;

use rand::prelude::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Default for TempDir {
    fn default() -> Self {
        let temp_dir = env::temp_dir();
        let mut ra = thread_rng();
        let path = temp_dir.join(&format!("rust-{}", ra.next_u32()));
        fs::create_dir(&path).unwrap();
        TempDir(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap()
    }
}
