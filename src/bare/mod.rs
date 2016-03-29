//! This module provides the core functionality
//! from the bare utility, in library form.
pub mod cli;
pub mod exit;
pub mod log;

use regex::Regex;
use std::collections::HashMap;
use std::path::{Path,PathBuf};

/// A Pattern object is a `(regex, replacement)` tuple.
/// The regex is used to match against files, and
/// replacement is the replacement text.
pub type Pattern = (Regex, String);

/// A Rename object is a `(src, dst)` tuple,
/// where `src` and `dst` represent file names.
pub type Rename = (String, String);

///
///
// [PathBuf](https://doc.rust-lang.org/std/path/struct.PathBuf.html)
pub type Proposal = HashMap<PathBuf, Vec<Rename>>;

pub fn propose_renames(paths: &[&Path], patterns: &[Pattern])
                       -> (Proposal, Vec<PathBuf>) {
    let (mut proposal, mut files_not_found) = (HashMap::new(), vec![]);
    for src_path in paths.iter() {
        if !src_path.exists() {
            files_not_found.push(src_path.to_path_buf());
            continue
        }
        let src_name = src_path.file_name().unwrap()
            .to_str().unwrap().to_string();
        let mut dst_name = src_name.clone();
        for &(ref regex, ref replacement) in patterns.iter() {
            if regex.is_match(&dst_name) {
                dst_name = regex.replace_all(&dst_name, replacement.as_str());
            }
        }
        let parent = src_path.parent().unwrap().to_path_buf();
        let mut renames = proposal.get(&parent).unwrap_or(&vec![]).clone();
        renames.push( (src_name, dst_name) );
        proposal.insert(parent, renames);
    }
    (proposal, files_not_found)
}




#[cfg(test)]
mod tests {
    use bare;
    use regex::Regex;
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    #[test]
    fn propose_renames_basic() {
        let (paths, patterns) = (paths(), patterns());
        ensure_exist(&paths);
        let (proposal, files_not_found) =
            bare::propose_renames(&paths, &patterns);
        assert_eq!(files_not_found, vec![] as Vec<PathBuf>);
        assert_eq!(proposal, {
            let mut m = HashMap::new();
            m.insert(PathBuf::from("/tmp/bare_test/"), vec![
                ("shooshoo.bar".to_string(),    "booboo.bar".to_string()),
                ("foo-bar.qux".to_string(),     "foo.coconut.qux".to_string()),
                ("_(grault).qux".to_string(),   ".grault.qux".to_string()),
            ]);
            m
        });
    }

    #[cfg(unix)]
    fn paths<'l>() -> Vec<&'l Path> {
        vec![
            Path::new("/tmp/bare_test/shooshoo.bar"),
            Path::new("/tmp/bare_test/foo-bar.qux"),
            Path::new("/tmp/bare_test/_(grault).qux"),
        ]
    }

    #[cfg(windows)]
    fn paths<'l>() -> Vec<&'l Path> {
        vec![
            // TODO:
        ]
    }

    fn patterns() -> Vec<(Regex, String)> {
        vec![
            (regex(r"shoo"),          replacement("boo")),
            (regex(r"(-)bar"),        replacement("_coconut")),
            (regex(r"\(grault\)"),    replacement("grault")),
            (regex(r"_"),             replacement(".")),
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

//  LocalWords:  PathBuf
