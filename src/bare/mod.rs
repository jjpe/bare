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




#[cfg(test)]
mod tests {
    use bare;
    use regex::Regex;
    use std::fs;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    #[test]
    fn propose_renames_basic() {
        let path_strings = file_path_strings();
        let paths: Vec<_> = path_strings.iter()
            .map(|s| Path::new(s))
            .collect();
        let patterns = vec![
            (regex(r"(-)bar"),        replacement("_coconut")),
            (regex(r"\(grault\)"),    replacement("grault")),
            (regex(r"_"),             replacement(".")),
        ];
        ensure_exist(&paths);
        let proposal = bare::propose_renames(&paths, &patterns);
        assert_eq!(proposal.not_found, vec![] as Vec<String>);
        assert_eq!(proposal.renames, vec![
            (PathBuf::from("/tmp/bare_test/foo.bar"),   // Old path, and ...
             PathBuf::from("/tmp/bare_test/foo.bar")),  // ...proposed new path
            (PathBuf::from("/tmp/bare_test/foo-bar.qux"),
             PathBuf::from("/tmp/bare_test/foo.coconut.qux")),
            (PathBuf::from("/tmp/bare_test/corge_(grault).quux"),
             PathBuf::from("/tmp/bare_test/corge.grault.quux"))
        ]);
    }

    #[cfg(unix)]
    fn file_path_strings<'a>() -> Vec<&'a str> {
        vec![
            "/tmp/bare_test/foo.bar",
            "/tmp/bare_test/foo-bar.qux",
            "/tmp/bare_test/corge_(grault).quux",
        ]
    }

    #[cfg(windows)]
    fn file_path_strings<'a>() -> Vec<&'a str> {
        vec![
            // TODO:
        ]
    }

    fn regex(literal: &'static str) -> Regex {
        Regex::new(literal).unwrap()
    }

    fn replacement(literal: &'static str) -> String {
        String::from(literal)
    }

    fn ensure_exist(paths: &[&Path]) {
        for path in paths {
            if path.exists() {  continue;  }
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            File::create(path).unwrap();
        }
    }
}
