use crate::{Dir, DirEntry};
use glob::{Pattern, PatternError};

impl<'a> Dir<'a> {
    /// Search for a file or directory with a glob pattern.
    pub fn find<'s>(&'s self, glob: &str) -> Result<impl Iterator<Item = &'s DirEntry<'a>>, PatternError> {
        let pattern = Pattern::new(glob)?;

        Ok(Globs::new(pattern, self))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Globs<'a, 'b> {
    stack: Vec<&'b DirEntry<'a>>,
    pattern: Pattern,
}

impl<'a, 'b> Globs<'a, 'b> {
    pub(crate) fn new<'r>(pattern: Pattern, root: &'r Dir<'a>) -> Globs<'a, 'r>
    {
        let stack = root.entries().iter().collect();
        Globs { stack, pattern }
    }
}

impl<'a, 'b> Iterator for Globs<'a, 'b> {
    type Item = &'b DirEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.pop() {
            self.stack.extend(item.children().iter());

            if self.pattern.matches_path(item.path()) {
                return Some(item);
            }
        }

        None
    }
}
