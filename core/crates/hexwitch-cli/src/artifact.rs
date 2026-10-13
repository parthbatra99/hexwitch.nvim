use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "artifact destination is a symbolic link",
        ));
    }
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "artifact requires a filename")
    })?;
    for _ in 0..32 {
        let temp_path = parent.join(format!(
            ".{}.{}-{}.tmp",
            name.to_string_lossy(),
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = match options.open(&temp_path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        let guard = Temporary(temp_path);
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&guard.0, path)?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot reserve artifact temporary file",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_complete_artifact_and_rejects_symlinks() {
        let dir = std::env::temp_dir().join(format!("hexwitch-atomic-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("result.json");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        #[cfg(unix)]
        {
            let link = dir.join("link.json");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(write_atomic(&link, b"bad").is_err());
            fs::remove_file(link).unwrap();
        }
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
