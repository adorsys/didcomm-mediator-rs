use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

// Define a trait for file system operations
pub trait FileSystem: Send + 'static {
    fn read_to_string(&self, path: &str) -> IoResult<String>;
    fn write(&mut self, path: &str, content: &str) -> IoResult<()>;
    fn read_dir_files(&self, path: &str) -> IoResult<Vec<String>>;
    fn create_dir_all(&mut self, path: &str) -> IoResult<()>;
    // Add other file system operations as needed
    fn open_with_options(
        &self,
        path: &str,
        read: bool,
        write: bool,
        create: bool,
    ) -> IoResult<Box<dyn FileHandle>>;
}

pub trait FileHandle {
    fn as_raw_fd(&self) -> i32;
}

// Implementation for StdFileHandle
struct StdFileHandle {
    file: File,
}

impl FileHandle for StdFileHandle {
    fn as_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
}

// Implement the trait for the actual file system
#[derive(Clone, Copy, Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_to_string(&self, path: &str) -> IoResult<String> {
        std::fs::read_to_string(path)
    }

    fn write(&mut self, path: &str, content: &str) -> IoResult<()> {
        std::fs::write(path, content)
    }

    fn read_dir_files(&self, path: &str) -> IoResult<Vec<String>> {
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

    fn create_dir_all(&mut self, path: &str) -> IoResult<()> {
        std::fs::create_dir_all(path)
    }

    fn open_with_options(
        &self,
        path: &str,
        read: bool,
        write: bool,
        create: bool,
    ) -> IoResult<Box<dyn FileHandle>> {
        let mut options = OpenOptions::new();
        options.read(read);
        options.write(write);
        options.create(create);

        let file = options.open(path)?;

        Ok(Box::new(StdFileHandle { file }))
    }

    // Implement other file system operations as needed
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
        fn read_to_string(&self, path: &str) -> IoResult<String> {
            Ok(self.map.get(path).cloned().unwrap_or_default())
        }

        fn write(&mut self, path: &str, content: &str) -> IoResult<()> {
            self.map.insert(path.to_string(), content.to_string());
            Ok(())
        }

        fn read_dir_files(&self, _path: &str) -> IoResult<Vec<String>> {
            Ok(vec![])
        }

        fn create_dir_all(&mut self, _path: &str) -> IoResult<()> {
            Ok(())
        }

        fn open_with_options(
            &self,
            path: &str,
            _read: bool,
            _write: bool,
            _create: bool,
        ) -> IoResult<Box<dyn FileHandle>> {
            // Mock implementation for open_with_options, not required for this test
            Ok(Box::new(MockFileHandle {
                content: self.map.get(path).cloned().unwrap_or_default(),
            }))
        }
    }

    struct MockFileHandle {
        content: String,
    }

    impl FileHandle for MockFileHandle {
        fn as_raw_fd(&self) -> i32 {
            // Not required for this test
            0
        }
    }

    #[test]
    fn can_mock_fs_operations() {
        let mut mock_fs = MockFileSystem::default();

        let res = mock_fs.write("/file.txt", "2456535e-a316-4d9e-8ab4-74a33d75d1fa");
        assert!(res.is_ok());

        let content = mock_fs.read_to_string("/file.txt").unwrap();
        assert_eq!(&content, "2456535e-a316-4d9e-8ab4-74a33d75d1fa");
    }
}
