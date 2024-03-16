use crate::{file::File, my_cow::Cow, DirEntry};
use std::fs;
use std::path::Path;

/// A directory.
#[derive(Debug, Clone, PartialEq)]
pub struct Dir<'a> {
    path: Cow<'a, str>,
    entries: Cow<'a, [DirEntry<'a>]>,
}

impl<'a> Dir<'a> {
    /// Create a new [`Dir`].
    pub const fn new(path: &'a str, entries: &'a [DirEntry<'a>]) -> Self {
        Dir {
            path: Cow::Borrowed(path),
            entries: Cow::Borrowed(entries),
        }
    }

    #[cfg(feature = "fs")]
    /// Create a new [`Dir`] by reading a filesystem directory (recursively) into memory
    pub fn from_fs(path: impl AsRef<Path>) -> std::io::Result<Dir<'static>> {
        let dir = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())?;
        Self::from_fs_and_path(String::new(), dir)
    }

    #[cfg(feature = "fs")]
    /// Create a new [`Dir`] by reading a filesystem directory (recursively) into memory
    pub fn from_fs_and_path(path: String, dir: cap_std::fs::Dir) -> std::io::Result<Dir<'static>> {
        Ok(Dir {
            path: Cow::Borrowed(""),
            entries: dir.entries()?.map(|entry| {
                let entry = entry?;
                let file_name = entry.file_name();
                let file_name = file_name.to_str().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Unsupported, "Filename contains non-utf8 characters")
                })?;
                let path = format!("{}/{}", path, file_name);
                DirEntry::from_fs(path, entry)
            }).collect::<Result<Vec<_>, _>>()
              .map(|vec| Cow::Owned(vec))?,
        })
    }


    /// The full path for this [`Dir`], relative to the directory passed to
    /// [`crate::include_dir!()`].
    pub fn path(&self) -> &Path {
        Path::new(self.path.as_ref())
    }

    /// The entries within this [`Dir`].
    pub fn entries(&self) -> &[DirEntry<'a>] {
        self.entries.as_ref()
    }

    /// Get a list of the files in this directory.
    pub fn files(&self) -> impl Iterator<Item = &File<'a>> {
        self.entries().iter().filter_map(DirEntry::as_file)
    }

    /// Get a list of the sub-directories inside this directory.
    pub fn dirs(&self) -> impl Iterator<Item = &Dir<'a>> {
        self.entries().iter().filter_map(DirEntry::as_dir)
    }

    /// Recursively search for a [`DirEntry`] with a particular path.
    pub fn get_entry<S: AsRef<Path>>(&self, path: S) -> Option<&DirEntry<'a>> {
        let path = path.as_ref();

        for entry in self.entries() {
            if entry.path() == path {
                return Some(entry);
            }

            if let DirEntry::Dir(d) = entry {
                if let Some(nested) = d.get_entry(path) {
                    return Some(nested);
                }
            }
        }

        None
    }

    /// Look up a file by name.
    pub fn get_file<S: AsRef<Path>>(&self, path: S) -> Option<&File<'a>> {
        self.get_entry(path).and_then(DirEntry::as_file)
    }

    /// Look up a dir by name.
    pub fn get_dir<S: AsRef<Path>>(&self, path: S) -> Option<&Dir<'a>> {
        self.get_entry(path).and_then(DirEntry::as_dir)
    }

    /// Does this directory contain `path`?
    pub fn contains<S: AsRef<Path>>(&self, path: S) -> bool {
        self.get_entry(path).is_some()
    }

    /// Create directories and extract all files to real filesystem.
    /// Creates parent directories of `path` if they do not already exist.
    /// Fails if some files already exist.
    /// In case of error, partially extracted directory may remain on the filesystem.
    pub fn extract<S: AsRef<Path>>(&self, base_path: S) -> std::io::Result<()> {
        let base_path = base_path.as_ref();

        for entry in self.entries() {
            let path = base_path.join(entry.path());

            match entry {
                DirEntry::Dir(d) => {
                    fs::create_dir_all(&path)?;
                    d.extract(base_path)?;
                }
                DirEntry::File(f) => {
                    fs::write(path, f.contents())?;
                }
            }
        }

        Ok(())
    }
}
