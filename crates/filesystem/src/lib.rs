use nix::fcntl::{flock, FlockArg};
use std::{
    fs::OpenOptions,
    io::{Error as IoError, ErrorKind, Result as IoResult},
    os::unix::io::AsRawFd,
    path::Path,
};

#[doc(hidden)]
// Define a trait for file system operations
pub trait FileSystem: Send + 'static {
    fn read_to_string(&self, path: &Path) -> IoResult<String>;
    fn write(&mut self, path: &Path, content: &str) -> IoResult<()>;
    fn read_dir_files(&self, path: &Path) -> IoResult<Vec<String>>;
    fn create_dir_all(&mut self, path: &Path) -> IoResult<()>;
    fn write_with_lock(&self, path: &Path, content: &str) -> IoResult<()>;
    // Add other file system operations as needed
}

// Implement the trait for the actual file system
#[derive(Clone, Copy, Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_to_string(&self, path: &Path) -> IoResult<String> {
        std::fs::read_to_string(path)
    }

    fn write(&mut self, path: &Path, content: &str) -> IoResult<()> {
        std::fs::write(path, content)
    }

    fn read_dir_files(&self, path: &Path) -> IoResult<Vec<String>> {
        let mut files = vec![];
        for entry in std::fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() {
                files.push(
                    path.to_str()
                        .ok_or(IoError::new(ErrorKind::Other, "InvalidPath"))?
                        .to_string(),
                )
            }
        }

        Ok(files)
    }

    fn create_dir_all(&mut self, path: &Path) -> IoResult<()> {
        std::fs::create_dir_all(path)
    }

    fn write_with_lock(&self, path: &Path, content: &str) -> IoResult<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        // Acquire an exclusive lock before writing to the file
        flock(file.as_raw_fd(), FlockArg::LockExclusive)
            .map_err(|_| IoError::new(ErrorKind::Other, "Error acquiring file lock"))?;

        std::fs::write(path, &content).expect("Error saving base64-encoded image to file");

        // Release the lock after writing to the file
        flock(file.as_raw_fd(), FlockArg::Unlock).expect("Error releasing file lock");
        Ok(())
    }

    // Implement other file system operations as needed
}

#[cfg(any(test, feature = "test-utils"))]
#[derive(Default)]
pub struct MockFileSystem;

#[cfg(any(test, feature = "test-utils"))]
impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &Path) -> IoResult<String> {
        match path {
            p if p.ends_with("did.json") => {
                Ok(include_str!("../test/storage/did.json").to_string())
            }
            _ => Err(IoError::new(ErrorKind::NotFound, "NotFound")),
        }
    }

    fn write(&mut self, _path: &Path, _content: &str) -> IoResult<()> {
        Ok(())
    }

    fn read_dir_files(&self, _path: &Path) -> IoResult<Vec<String>> {
        Ok(vec!["/secrets.json".to_string()])
    }

    fn create_dir_all(&mut self, _path: &Path) -> IoResult<()> {
        Ok(())
    }

    fn write_with_lock(&self, _path: &Path, _content: &str) -> IoResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    // Now, for testing, create a mock implementation of the trait
    #[derive(Default)]
    struct MockFileSystem {
        map: HashMap<String, String>,
    }

    impl FileSystem for MockFileSystem {
        fn read_to_string(&self, path: &Path) -> IoResult<String> {
            Ok(self.map.get(path.to_str().unwrap()).cloned().unwrap_or_default())
        }

        fn write(&mut self, path: &Path, content: &str) -> IoResult<()> {
            self.map.insert(path.to_str().unwrap().to_string(), content.to_string());
            Ok(())
        }

        fn read_dir_files(&self, _path: &Path) -> IoResult<Vec<String>> {
            Ok(vec![])
        }

        fn create_dir_all(&mut self, _path: &Path) -> IoResult<()> {
            Ok(())
        }

        fn write_with_lock(&self, _path: &Path, _content: &str) -> IoResult<()> {
            Ok(())
        }
    }

    #[test]
    fn can_mock_fs_operations() {
        let mut mock_fs = MockFileSystem::default();

        let res = mock_fs.write("/file.txt".as_ref(), "2456535e-a316-4d9e-8ab4-74a33d75d1fa");
        assert!(res.is_ok());

        let content = mock_fs.read_to_string("/file.txt".as_ref()).unwrap();
        assert_eq!(&content, "2456535e-a316-4d9e-8ab4-74a33d75d1fa");
    }
}
