use std::fs;
use std::io;
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

use anyhow::{anyhow, Error, Result};
use lib::api;

pub(super) async fn list_directory(
    request: api::ListDirectory,
) -> Result<api::ListDirectoryResponse> {
    tokio::task::spawn_blocking(move || list(&request)).await?
}

fn list(request: &api::ListDirectory) -> Result<api::ListDirectoryResponse> {
    let user = directories::UserDirs::new();
    let home = user.as_ref().map(|user| user.home_dir().to_owned());

    let requested = if request.path.is_empty() {
        home.clone().ok_or_else(|| anyhow!("No home directory"))?
    } else {
        PathBuf::from(&request.path)
    };

    let (dir, file) = locate(&requested, request.nearest)?;

    let mut entries = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|e| io(&dir, e))? {
        let Ok(entry) = entry else {
            continue;
        };

        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };

        let (kind, size) = match fs::metadata(entry.path()) {
            Ok(m) if m.is_dir() => (api::DirectoryEntryKind::Directory, None),
            Ok(m) if m.is_file() => (api::DirectoryEntryKind::File, Some(m.len())),
            _ => (api::DirectoryEntryKind::Other, None),
        };

        entries.push(api::DirectoryEntry { name, kind, size });
    }

    entries.sort_by(|a, b| {
        let a_dir = a.kind == api::DirectoryEntryKind::Directory;
        let b_dir = b.kind == api::DirectoryEntryKind::Directory;
        b_dir
            .cmp(&a_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut places = Vec::new();

    let mut place = |kind, path: Option<&Path>| {
        if let Some(path) = path.filter(|p| p.is_dir()).and_then(|p| p.to_str()) {
            places.push(api::Place {
                kind,
                path: path.to_owned(),
            });
        }
    };

    place(api::PlaceKind::Home, home.as_deref());

    if let Some(user) = &user {
        place(api::PlaceKind::Documents, user.document_dir());
        place(api::PlaceKind::Downloads, user.download_dir());
    }

    for root in roots() {
        place(api::PlaceKind::Root, Some(&root));
    }

    Ok(api::ListDirectoryResponse {
        path: utf8(&dir)?,
        parent: dir.parent().map(utf8).transpose()?,
        separator: MAIN_SEPARATOR_STR.to_owned(),
        entries,
        file,
        places,
    })
}

/// The absolute directory to list for `path`, and the name in it `path`
/// named when that is not a directory.
fn locate(path: &Path, nearest: bool) -> Result<(PathBuf, Option<String>)> {
    let path = std::path::absolute(path).map_err(|e| io(path, e))?;
    let mut current = path.as_path();
    let mut file = None;

    loop {
        match fs::metadata(current) {
            Ok(m) if m.is_dir() => break,
            Ok(_) => {}
            Err(error) if nearest && error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(io(current, error)),
        }

        if file.is_none() {
            file = current
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_owned);
        }

        current = current
            .parent()
            .ok_or_else(|| anyhow!("{}: No such directory", path.display()))?;
    }

    let dir = fs::canonicalize(current).map_err(|e| io(current, e))?;
    Ok((dir, file))
}

/// The error with its path, in one message since only the outermost one
/// reaches the UI.
fn io(path: &Path, error: io::Error) -> Error {
    anyhow!("{}: {error}", path.display())
}

fn utf8(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("{}: Path is not valid UTF-8", path.display()))
}

#[cfg(unix)]
fn roots() -> Vec<PathBuf> {
    vec![PathBuf::from("/")]
}

#[cfg(windows)]
fn roots() -> Vec<PathBuf> {
    (b'A'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\", char::from(letter))))
        .filter(|root| root.is_dir())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use lib::api;

    use super::list;

    fn request(path: &std::path::Path, nearest: bool) -> api::ListDirectory {
        api::ListDirectory {
            path: path.to_str().unwrap().to_owned(),
            nearest,
        }
    }

    #[test]
    fn lists_directories_first() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("b.gz"), b"abc").unwrap();
        fs::write(dir.path().join("A.txt"), b"").unwrap();
        fs::create_dir(dir.path().join("z")).unwrap();

        let response = list(&request(dir.path(), false)).unwrap();

        let entries = response
            .entries
            .iter()
            .map(|e| (e.name.as_str(), e.kind, e.size))
            .collect::<Vec<_>>();

        assert_eq!(
            entries,
            [
                ("z", api::DirectoryEntryKind::Directory, None),
                ("A.txt", api::DirectoryEntryKind::File, Some(0)),
                ("b.gz", api::DirectoryEntryKind::File, Some(3)),
            ]
        );

        assert_eq!(response.file, None);
        assert!(response.parent.is_some());
    }

    #[test]
    fn file_lists_its_directory() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("JMdict_e.gz");
        fs::write(&file, b"").unwrap();

        let response = list(&request(&file, false)).unwrap();
        let expected = fs::canonicalize(dir.path()).unwrap();
        assert_eq!(response.path, expected.to_str().unwrap());
        assert_eq!(response.file.as_deref(), Some("JMdict_e.gz"));
    }

    #[test]
    fn missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing").join("out.gz");

        let error = list(&request(&missing, false)).unwrap_err();
        assert!(error.to_string().contains("missing"));
        assert!(error.to_string().contains("No such file"));

        let response = list(&request(&missing, true)).unwrap();
        let expected = fs::canonicalize(dir.path()).unwrap();
        assert_eq!(response.path, expected.to_str().unwrap());
        assert_eq!(response.file.as_deref(), Some("out.gz"));
    }
}
