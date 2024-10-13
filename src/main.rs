use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use ignore::gitignore::Gitignore;

fn main() -> anyhow::Result<()> {
    // TODO: add command line argument parsing:
    //  - ignore said file (optional recursive)
    //  - unignore said file (optional recursive)
    //  - output statistics to stdout
    //  - dry-run

    if std::env::args().any(|arg| arg == "--dry-run") {
        let folder = std::env::args().nth(2).expect("Please provide a folder");
        traverse_folder::<DryRunAttributes>(&Path::new(&folder))?;
    } else {
        let folder = std::env::args().nth(1).expect("Please provide a folder");
        traverse_folder::<FileSystemAttributes>(&Path::new(&folder))?;
    }

    Ok(())
}

fn traverse_folder<A: AttributesIO>(folder: &Path) -> anyhow::Result<()> {
    let folder = std::path::absolute(folder)?;
    assert!(folder.is_dir());
    assert!(folder.exists());

    let mut todo: VecDeque<(Vec<Gitignore>, PathBuf)> = VecDeque::new();
    todo.push_back((Vec::new(), folder));

    let mut count = 0;

    while let Some((mut ignores, path)) = todo.pop_front() {
        assert!(path.is_dir(), "should only iterate over directories");
        assert!(
            path.is_absolute(),
            "should only iterate over absolute paths"
        );

        // first check if there is a .dropboxignore file in this directory
        // if so, read it and add it to the list of ignorers
        let ignorefile = path.join(".dropboxignore");
        if ignorefile.exists() {
            let (ignore, error) = Gitignore::new(&ignorefile);
            if let Some(e) = error {
                bail!(
                    "Error reading .dropboxignore file {:?}: {:?}",
                    ignorefile,
                    e
                );
            }
            ignores.push(ignore);
        }

        // iterate over all files in the directory
        for entry in path.read_dir()? {
            let entry = entry?;
            let path = entry.path();

            let is_dir = path.is_dir();

            let is_ignored = is_file_ignored(&path)?;

            // check if file matches any of the ignores
            if ignores
                .iter()
                .any(|ignore| ignore.matched(&path, is_dir).is_ignore())
            {
                if !is_ignored {
                    println!("ignoring {:?}", path);
                    A::ignore_file(&path)?;
                }

                // since the file/folder is supposed to be ignored, we don't need to check it's children
                continue;
            } else {
                //if file is ignored already, should maybe unignore it?
                //Or should we policy that if a file is ignored, it should stay ignored? (perhaps
                //easies in the beginning)
                if is_ignored {
                    println!(
                        "file {:?} is ignored but it should not be according to the rules",
                        path
                    );
                }
            }

            // traverse into the sub-directory
            if is_dir {
                todo.push_back((ignores.clone(), path));
            } else {
                // this was a non-ignored file, count!
                // (We can add measure of size here if we want to)
                count += 1;
            }
        }
    }

    dbg!(count);

    Ok(())
}

/// Trait for reading and writing attributes to a file
trait AttributesIO {
    /// Set the file to be ignored
    fn ignore_file(file: &Path) -> Result<()>;
}

struct FileSystemAttributes;
impl FileSystemAttributes {
    const XATTR_DROPBOX_IGNORED: &str = "user.com.dropbox.ignored";
}

fn is_file_ignored(file: &Path) -> Result<bool> {
    let attr = xattr::get(file, FileSystemAttributes::XATTR_DROPBOX_IGNORED)
        .with_context(|| format!("get attribute for {file:?}"))?;
    Ok(attr.map(|attr| attr == b"1").unwrap_or(false))
}

impl AttributesIO for FileSystemAttributes {
    fn ignore_file(file: &Path) -> Result<()> {
        xattr::set(file, Self::XATTR_DROPBOX_IGNORED, b"1")
            .with_context(|| format!("set attribute for {file:?}"))?;
        Ok(())
    }
}

struct DryRunAttributes;
impl AttributesIO for DryRunAttributes {
    fn ignore_file(file: &Path) -> Result<()> {
        println!("DRYRUN: ignore {:?}", file);
        Ok(())
    }
}
