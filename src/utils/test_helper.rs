use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{fs, io};

// one possible implementation of walking a directory only visiting files
pub fn list_files(
    dir: &Path,
    buff: &mut Vec<PathBuf>,
    filter: &mut dyn Fn(&DirEntry) -> bool,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                list_files(&path, buff, filter)?;
            } else if filter(&entry) {
                buff.push(path)
            }
        }
    }
    Ok(())
}
