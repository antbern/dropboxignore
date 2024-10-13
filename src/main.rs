use std::{collections::VecDeque, path::Path};

fn main() -> anyhow::Result<()> {
    let folder = std::env::args().nth(1).expect("Please provide a folder");
    traverse_folder(&Path::new(&folder))?;

    Ok(())
}

fn traverse_folder(folder: &Path) -> anyhow::Result<()> {
    let mut todo = VecDeque::new();
    todo.push_back(folder.to_path_buf());

    while let Some(path) = todo.pop_front() {
        if path.is_dir() {
            for entry in path.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                todo.push_back(path);
            }
        } else {
            println!("{}", path.display());
        }
    }

    Ok(())
}
