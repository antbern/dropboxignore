use std::{collections::VecDeque, fs::File, path::Path};

use anyhow::{Context, Result};

fn main() -> anyhow::Result<()> {
    let folder = std::env::args().nth(1).expect("Please provide a folder");

    traverse_folder(&Path::new(&folder))?;

    Ok(())
}

fn traverse_folder(folder: &Path) -> anyhow::Result<()> {
    let folder = std::path::absolute(folder)?;
    assert!(folder.is_dir());
    assert!(folder.exists());

    let mut todo = VecDeque::new();
    todo.push_back(folder);


    let mut count = 0;

    while let Some(path) = todo.pop_front() {
        if is_ignored(&path)? {
            continue;
        }
        if path.is_dir() {
            for entry in path.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                todo.push_back(path);
            }
        } else {
            count += 1;
            // println!("{}; {}", is_ignored(&path)?, path.display());
        }
    }

    dbg!(count);

    Ok(())
}

const XATTR_DROPBOX_IGNORED: &str = "user.com.dropbox.ignored";
fn is_ignored(file: &Path) -> Result<bool> {
    let attr =
        xattr::get(file, XATTR_DROPBOX_IGNORED).with_context(|| format!("get attribute for {file:?}"))?;
    Ok(attr.map(|attr| attr == b"1").unwrap_or(false))
}

fn ignore_file(file: &Path) -> Result<()> {
    todo!();
}
