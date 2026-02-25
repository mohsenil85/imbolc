use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// File browser bookmarks: single-char keys mapped to directory paths.
/// Persisted as TOML at `~/.config/imbolc/bookmarks.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Bookmarks {
    #[serde(default)]
    entries: BTreeMap<String, PathBuf>,
}

impl Bookmarks {
    pub fn load() -> Self {
        let path = Self::storage_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(toml_str) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, toml_str);
        }
    }

    pub fn get(&self, key: &char) -> Option<&PathBuf> {
        self.entries.get(&key.to_string())
    }

    pub fn set(&mut self, key: char, path: PathBuf) {
        self.entries.insert(key.to_string(), path);
    }

    pub fn delete(&mut self, key: char) -> bool {
        self.entries.remove(&key.to_string()).is_some()
    }

    /// Returns an iterator of (char, &PathBuf) pairs, sorted by key.
    pub fn iter(&self) -> impl Iterator<Item = (char, &PathBuf)> {
        self.entries.iter().filter_map(|(k, v)| {
            let ch = k.chars().next()?;
            if k.len() == 1 {
                Some((ch, v))
            } else {
                None
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn storage_path() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("imbolc")
                .join("bookmarks.toml")
        } else {
            PathBuf::from("bookmarks.toml")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_operations() {
        let mut bookmarks = Bookmarks::default();

        // Set
        bookmarks.set('s', PathBuf::from("/samples"));
        bookmarks.set('p', PathBuf::from("/projects"));
        assert_eq!(bookmarks.get(&'s'), Some(&PathBuf::from("/samples")));
        assert_eq!(bookmarks.get(&'p'), Some(&PathBuf::from("/projects")));
        assert_eq!(bookmarks.get(&'x'), None);

        // Overwrite
        bookmarks.set('s', PathBuf::from("/new-samples"));
        assert_eq!(bookmarks.get(&'s'), Some(&PathBuf::from("/new-samples")));

        // Delete
        assert!(bookmarks.delete('s'));
        assert_eq!(bookmarks.get(&'s'), None);
        assert!(!bookmarks.delete('s')); // already gone

        // Iter
        let entries: Vec<(char, &PathBuf)> = bookmarks.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, 'p');
    }

    #[test]
    fn serialization_roundtrip() {
        let mut bookmarks = Bookmarks::default();
        bookmarks.set('s', PathBuf::from("/samples"));
        bookmarks.set('p', PathBuf::from("/projects"));

        let toml_str = toml::to_string_pretty(&bookmarks).unwrap();
        let loaded: Bookmarks = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.get(&'s'), Some(&PathBuf::from("/samples")));
        assert_eq!(loaded.get(&'p'), Some(&PathBuf::from("/projects")));
    }

    #[test]
    fn empty_bookmarks_serialize() {
        let bookmarks = Bookmarks::default();
        let toml_str = toml::to_string_pretty(&bookmarks).unwrap();
        let loaded: Bookmarks = toml::from_str(&toml_str).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn iter_is_sorted() {
        let mut bookmarks = Bookmarks::default();
        bookmarks.set('z', PathBuf::from("/z"));
        bookmarks.set('a', PathBuf::from("/a"));
        bookmarks.set('m', PathBuf::from("/m"));

        let keys: Vec<char> = bookmarks.iter().map(|(ch, _)| ch).collect();
        assert_eq!(keys, vec!['a', 'm', 'z']);
    }
}
