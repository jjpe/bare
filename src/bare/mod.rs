//! This module provides the core functionality
//! from the bare utility, in library form.
pub mod cli;
pub mod exit;
pub mod log;

use regex::Regex;
use std::path::{Path,PathBuf};

pub type Pattern = (Regex, String);

#[derive(Debug)]
pub struct RenameProposal {
    pub renames: Vec<(PathBuf, PathBuf)>,
    pub not_found: Vec<String>
}

pub fn propose_renames(paths: &[&Path],
                       patterns: &[Pattern]) -> RenameProposal {
    let mut renames: Vec<(PathBuf, PathBuf)> = vec![];
    let mut not_found: Vec<String> = vec![];
    for src_path in paths.iter() {
        let parent: &Path = src_path.parent().unwrap();
        let src_name: &str =
            src_path.file_name().unwrap().to_str().unwrap();
        if !src_path.exists() {
            not_found.push(String::from(src_path.to_str().unwrap()));
            continue
        }
        let mut dst_name = String::from(src_name);
        for pat in patterns.iter() {
            let (regex, replacement): (&Regex, &str) = (&pat.0, &pat.1);
            if regex.is_match(&dst_name) {
                dst_name = regex.replace(&dst_name, replacement);
            }
        }
        let dst_path = Path::new(parent.to_str().unwrap())
            .join(Path::new(&dst_name));
        renames.push((src_path.to_path_buf(), dst_path));
    }
    RenameProposal { renames: renames, not_found: not_found }
}
