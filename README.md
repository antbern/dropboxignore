# dropboxignore

[![CI](https://github.com/antbern/dropboxignore/actions/workflows/rust.yaml/badge.svg)](https://github.com/antbern/dropboxignore/actions/workflows/rust.yaml)

A small utility for dealing with ignored files when using `dropbox` on Linux.
It introduces a simple `.dropboxignore` with the same contents as a standard `.gitignore` file
that can be placed anywhere in the `dropbox` sync folder. When this program is run with the
`check` command and a path to the `dropbox` sync folder, it will traverse the folders and
match each file against the found `.dropboxignore` files (respecting all files in the parent
folders as well), and update the `com.dropbox.ignored` extended file attribute to tell `dropbox` to
ignore (eg. not sync) the specified file or folder.

The idea is that the `dropboxignore` tool shall be run periodically, eg. every minute, to keep the
right files and folders ignored while you work.

As an example of a `.dropboxignore` file, see the one [in this repo](./.dropboxignore). Very useful
for keeping `dropbox` from syncing the rust `target/` folder containing build artifacts.

## Usage
```
Usage: dropboxignore <COMMAND> [FLAGS] <FOLDER SYNCED BY DROPBOX>

Commands:
  check      Traverse and ignore files as depicted by the .dropboxignore files.
  ignore     Ignore the specified file/folder.
  unignore   Unignore the specified file/folder.

Options:
  --dry-run        Do not apply any changes to the file system, only print out what it
                   would do if run without the flag.
  -r, --recursive  Apply ignore and unignore commands to all files and subfolders of the
                   provided folder, recursively.
```
