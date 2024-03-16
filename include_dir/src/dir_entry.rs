use crate::{Dir, File};
use std::path::Path;

#[cfg(feature = "fs")]
use crate::my_cow::Cow;

#[cfg(feature = "fs")]
use std::io::Read;

/// A directory entry, roughly analogous to [`std::fs::DirEntry`].
#[derive(Debug, Clone, PartialEq)]
pub enum DirEntry<'a> {
    /// A directory.
    Dir(Dir<'a>),
    /// A file.
    File(File<'a>),
}

impl<'a> DirEntry<'a> {
    #[cfg(feature = "fs")]
    pub(crate) fn from_fs(path: String, entry: cap_std::fs::DirEntry) -> std::io::Result<DirEntry<'static>> {
        let kind = entry.file_type()?;
        if kind.is_file() {
            let mut contents = vec![];
            entry.open()?.read_to_end(&mut contents)?;
            Ok(DirEntry::File(File {
                path: Cow::Owned(path),
                contents: Cow::Owned(contents),
            }))
        }
        else if kind.is_dir() {
            Ok(DirEntry::Dir(Dir::from_fs_and_path(path, entry.open_dir()?)?))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Directory contains something that is neither a File nor a Directory"))
        }
    }

    /// The [`DirEntry`]'s full path.
    pub fn path(&self) -> &Path {
        match self {
            DirEntry::Dir(d) => d.path(),
            DirEntry::File(f) => f.path(),
        }
    }

    /// Try to get this as a [`Dir`], if it is one.
    pub fn as_dir(&self) -> Option<&Dir<'a>> {
        match self {
            DirEntry::Dir(d) => Some(d),
            DirEntry::File(_) => None,
        }
    }

    /// Try to get this as a [`File`], if it is one.
    pub fn as_file(&self) -> Option<&File<'a>> {
        match self {
            DirEntry::File(f) => Some(f),
            DirEntry::Dir(_) => None,
        }
    }

    /// Get this item's sub-items, if it has any.
    pub fn children(&self) -> &[DirEntry<'a>] {
        match self {
            DirEntry::Dir(d) => d.entries(),
            DirEntry::File(_) => &[],
        }
    }
}
