use std::{
    collections::VecDeque,
    os::linux::fs::MetadataExt,
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

    let args = std::env::args().collect::<Vec<_>>();

    // remove the executable name
    let args: &[String] = &args[1..];

    // command is the first argument
    let (command, args) = args.split_first().expect("Please provide a command");
    // folder is the last argument, flags are in between
    let (folder, flags) = args
        .split_last()
        .expect("Please provide a folder to operate on");

    let folder = Path::new(&folder);
    if !folder.is_dir() {
        bail!("provided path {folder:?} is not a directory or it does not exist");
    }

    match command.as_str() {
        "check" => {
            let mut is_dry_run = false;
            for flag in flags {
                match flag.as_str() {
                    "--dry-run" => {
                        is_dry_run = true;
                    }
                    _ => bail!("Unknown flag: {}", flag),
                }
            }

            if is_dry_run {
                traverse_folder::<DryRunAttributes>(folder)?;
            } else {
                traverse_folder::<FileSystemAttributes>(folder)?;
            }
        }
        _ => bail!("Unknown command: {}", command),
    }

    Ok(())
}

#[derive(Debug, Default)]
struct Stats {
    files: u64,
    directories: u64,
    size: u64,
}

fn traverse_folder<A: AttributesIO>(folder: &Path) -> anyhow::Result<()> {
    let folder = std::path::absolute(folder)?;
    assert!(folder.is_dir());
    assert!(folder.exists());

    let mut todo: VecDeque<(Vec<Gitignore>, PathBuf)> = VecDeque::new();
    todo.push_back((Vec::new(), folder));

    let mut stats = Stats::default();

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
                    A::ignore_file(&path)?;
                }

                // since the file/folder is supposed to be ignored, we don't need to check it's children
                continue;
            } else if is_ignored {
                //if file is ignored already, should maybe unignore it?
                //Or should we policy that if a file is ignored, it should stay ignored? (perhaps
                //easies in the beginning)
                println!(
                    "file {:?} is ignored but it should not be according to the rules",
                    path
                );
            } else {
                // file/folder is not ignored and should not be either

                // only count files, not symlinks for now
                let meta =
                    std::fs::symlink_metadata(&path).with_context(|| path.display().to_string())?;
                let size = meta.st_size(); // only works on linux

                stats.size += size;

                if is_dir {
                    stats.directories += 1;

                    // traverse into the sub-directory
                    todo.push_back((ignores.clone(), path));
                } else {
                    // this was a non-ignored file, count!
                    // (We can add measure of size here if we want to)

                    stats.files += 1;
                }
            }
        }
    }

    println!(
        "Stats:\nFiles: {}\nDirectories: {}\nSize: {:.2} MB, {:.2} GB",
        stats.files,
        stats.directories,
        stats.size as f64 / 1024.0 / 1024.0,
        stats.size as f64 / 1024.0 / 1024.0 / 1024.0
    );

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
        println!("ignoring {:?}", file);
        xattr::set(file, Self::XATTR_DROPBOX_IGNORED, b"1")
            .with_context(|| format!("set attribute for {file:?}"))?;
        Ok(())
    }
}

struct DryRunAttributes;
impl AttributesIO for DryRunAttributes {
    fn ignore_file(file: &Path) -> Result<()> {
        println!("DRYRUN: ignoring {:?}", file);
        Ok(())
    }
}
