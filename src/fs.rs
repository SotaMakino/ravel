use rquickjs::{Ctx, Error, Object, Result, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::encoding::bytes_from_value;

fn resolve_path(root: &Path, input: &str) -> std::io::Result<PathBuf> {
    if input.contains('\0') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "null byte in path",
        ));
    }

    let canon_root = root.canonicalize()?;

    let joined = root.join(input);
    let canon_resolved = joined.canonicalize()?;

    if !canon_resolved.starts_with(&canon_root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
    }

    Ok(canon_resolved)
}

fn resolve_path_for_write(root: &Path, input: &str) -> std::io::Result<PathBuf> {
    if input.contains('\0') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "null byte in path",
        ));
    }

    let canon_root = root.canonicalize()?;

    let joined = root.join(input);

    let canon_resolved = if joined.exists() {
        joined.canonicalize()?
    } else {
        let mut candidate = joined.clone();
        loop {
            if candidate.exists() {
                let canon_candidate = candidate.canonicalize()?;
                if !canon_candidate.starts_with(&canon_root) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "permission denied",
                    ));
                }
                break;
            }
            let parent = candidate.parent();
            if parent.is_none() || parent == Some(Path::new("")) {
                break;
            }
            candidate = parent.unwrap().to_path_buf();
        }

        joined
    };

    if !canon_resolved.starts_with(&canon_root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
    }

    Ok(canon_resolved)
}

pub fn setup_fs<'js>(ctx: &Ctx<'js>, root: &Path) -> Result<()> {
    let root1 = root.to_path_buf();
    let root2 = root.to_path_buf();
    let root3 = root.to_path_buf();
    let root4 = root.to_path_buf();
    let fs_obj = Object::new(ctx.clone())?;

    fs_obj.set(
        "readFile",
        rquickjs::function::Func::new(move |ctx: Ctx<'js>, path: String| -> Result<Value<'js>> {
            let resolved = resolve_path(&root1, &path).map_err(|e| {
                Error::new_from_js_message("string", "file", format!("{}: '{}'", e, path))
            })?;
            match fs::read(&resolved) {
                Ok(bytes) => {
                    let arr = rquickjs::ArrayBuffer::new(ctx.clone(), bytes)?;
                    Ok(rquickjs::TypedArray::<u8>::from_arraybuffer(arr)?.into_value())
                }
                Err(e) => Err(Error::new_from_js_message(
                    "string",
                    "file",
                    format!("{}: '{}'", e, resolved.display()),
                )),
            }
        }),
    )?;

    fs_obj.set(
        "writeFile",
        rquickjs::function::Func::new(
            move |path: String, data: Value<'js>| -> Result<()> {
                // Accept a string directly; bytes still work for binary output.
                let bytes = match data.as_string() {
                    Some(s) => s.to_string()?.into_bytes(),
                    None => bytes_from_value(&data).map_err(|_| {
                        Error::new_from_js_message(
                            "file",
                            "write",
                            format!("expected a string, Uint8Array, or ArrayBuffer: '{}'", path),
                        )
                    })?,
                };
                let resolved = resolve_path_for_write(&root2, &path).map_err(|e| {
                    Error::new_from_js_message("file", "write", format!("{}: '{}'", e, path))
                })?;
                if let Some(parent) = resolved.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        Error::new_from_js_message(
                            "file",
                            "write",
                            format!("failed to create directories: {}: '{}'", e, path),
                        )
                    })?;
                }
                fs::write(&resolved, &bytes).map_err(|e| {
                    Error::new_from_js_message(
                        "file",
                        "write",
                        format!("{}: '{}'", e, resolved.display()),
                    )
                })
            },
        ),
    )?;

    fs_obj.set(
        "exists",
        rquickjs::function::Func::new(move |path: String| -> bool {
            match resolve_path(&root3, &path) {
                Ok(resolved) => resolved.exists(),
                Err(_) => false,
            }
        }),
    )?;

    fs_obj.set(
        "mkdirSync",
        rquickjs::function::Func::new(move |path: String| -> Result<()> {
            let canon_root = root4.canonicalize().map_err(|e| {
                Error::new_from_js_message("file", "mkdir", format!("{}: '{}'", e, path))
            })?;
            let joined = root4.join(&path);
            let resolved = if joined.exists() {
                joined.canonicalize().map_err(|e| {
                    Error::new_from_js_message("file", "mkdir", format!("{}: '{}'", e, path))
                })?
            } else {
                let parent = joined.parent().unwrap_or(Path::new(""));
                if parent.exists() {
                    let canon_parent = parent.canonicalize().map_err(|e| {
                        Error::new_from_js_message("file", "mkdir", format!("{}: '{}'", e, path))
                    })?;
                    if !canon_parent.starts_with(&canon_root) {
                        return Err(Error::new_from_js_message(
                            "file",
                            "mkdir",
                            format!("permission denied: '{}'", path),
                        ));
                    }
                }
                joined
            };
            if !resolved.starts_with(&canon_root) {
                return Err(Error::new_from_js_message(
                    "file",
                    "mkdir",
                    format!("permission denied: '{}'", path),
                ));
            }
            fs::create_dir_all(&resolved).map_err(|e| {
                Error::new_from_js_message("file", "mkdir", format!("{}: '{}'", e, path))
            })
        }),
    )?;

    ctx.globals().set("fs", fs_obj)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn setup_test_dir() -> std::io::Result<(tempfile::TempDir, PathBuf)> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().to_path_buf();
        Ok((temp, root))
    }

    #[test]
    fn test_resolve_path_existing_file() {
        let (temp, root) = setup_test_dir().unwrap();
        let file_path = root.join("test.txt");
        let mut f = fs::File::create(&file_path).unwrap();
        f.write_all(b"hello").unwrap();
        drop(f);

        let resolved = resolve_path(&root, "test.txt").unwrap();
        assert_eq!(resolved, file_path.canonicalize().unwrap());
        drop(temp);
    }

    #[test]
    fn test_resolve_path_nested_file() {
        let (temp, root) = setup_test_dir().unwrap();
        let sub_dir = root.join("sub");
        fs::create_dir(&sub_dir).unwrap();
        let file_path = sub_dir.join("nested.txt");
        let mut f = fs::File::create(&file_path).unwrap();
        f.write_all(b"nested").unwrap();
        drop(f);

        let resolved = resolve_path(&root, "sub/nested.txt").unwrap();
        assert_eq!(resolved, file_path.canonicalize().unwrap());
        drop(temp);
    }

    #[test]
    fn test_resolve_path_escape_attempt() {
        let (_temp, root) = setup_test_dir().unwrap();
        let result = resolve_path(&root, "../outside.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_path_absolute_escape() {
        let (_temp, root) = setup_test_dir().unwrap();
        let result = resolve_path(&root, "/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_path_for_write_existing() {
        let (temp, root) = setup_test_dir().unwrap();
        let file_path = root.join("write.txt");
        let mut f = fs::File::create(&file_path).unwrap();
        f.write_all(b"existing").unwrap();
        drop(f);

        let resolved = resolve_path_for_write(&root, "write.txt").unwrap();
        assert_eq!(resolved, file_path.canonicalize().unwrap());
        drop(temp);
    }

    #[test]
    fn test_resolve_path_for_write_new_file() {
        let (temp, root) = setup_test_dir().unwrap();
        let existing = root.join("existing.txt");
        fs::File::create(&existing).unwrap();
        let canon_root = root.canonicalize().unwrap();
        let resolved = resolve_path_for_write(&canon_root, "new_file.txt").unwrap();
        assert_eq!(resolved, canon_root.join("new_file.txt"));
        drop(temp);
    }

    #[test]
    fn test_resolve_path_for_write_escape_attempt() {
        let (_temp, root) = setup_test_dir().unwrap();
        let result = resolve_path_for_write(&root, "../escape.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_path_for_write_new_in_subdir() {
        let (temp, root) = setup_test_dir().unwrap();
        let sub_dir = root.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        let existing = sub_dir.join("existing.txt");
        fs::File::create(&existing).unwrap();
        let canon_root = root.canonicalize().unwrap();
        let resolved = resolve_path_for_write(&canon_root, "subdir/new.txt").unwrap();
        assert_eq!(resolved, canon_root.join("subdir/new.txt"));
        drop(temp);
    }

    #[test]
    fn test_resolve_path_symlink_escape() {
        let (temp, root) = setup_test_dir().unwrap();
        let outside_temp = tempfile::tempdir().unwrap();
        let outside = outside_temp.path().join("outside.txt");
        fs::write(&outside, "secret").unwrap();
        let link = root.join("escape_link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside, &link).unwrap();
        let result = resolve_path(&root, "escape_link");
        assert!(result.is_err(), "symlink escape should be denied");
        drop(outside_temp);
        drop(temp);
    }

    #[test]
    fn test_resolve_path_symlink_allowed_within_root() {
        let (temp, root) = setup_test_dir().unwrap();
        let target = root.join("target.txt");
        fs::write(&target, "hello").unwrap();
        let link = root.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &link).unwrap();
        let result = resolve_path(&root, "link");
        assert!(result.is_ok(), "symlink within root should be allowed");
        drop(temp);
    }

    #[test]
    fn test_resolve_path_for_write_symlink_escape() {
        let (temp, root) = setup_test_dir().unwrap();
        let outside_temp = tempfile::tempdir().unwrap();
        let outside_dir = outside_temp.path().join("outside_dir");
        fs::create_dir(&outside_dir).unwrap();
        let link = root.join("escape_link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside_dir, &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&outside_dir, &link).unwrap();
        let result = resolve_path_for_write(&root, "escape_link/file.txt");
        assert!(result.is_err(), "symlink write escape should be denied");
        drop(outside_temp);
        drop(temp);
    }

    #[test]
    fn test_resolve_path_null_byte_rejection() {
        let (_temp, root) = setup_test_dir().unwrap();
        let result = resolve_path(&root, "file\0.txt");
        assert!(result.is_err(), "null byte in path should be rejected");
    }

    #[test]
    fn test_resolve_path_for_write_null_byte_rejection() {
        let (_temp, root) = setup_test_dir().unwrap();
        let result = resolve_path_for_write(&root, "file\0.txt");
        assert!(result.is_err(), "null byte in write path should be rejected");
    }

    #[test]
    fn test_setup_fs() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = rquickjs::Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            let root = std::env::current_dir().unwrap();
            setup_fs(&ctx, &root).unwrap();
            let fs_obj: rquickjs::Object = ctx.globals().get("fs").unwrap();
            assert!(fs_obj.get::<_, rquickjs::Function>("readFile").is_ok());
            assert!(fs_obj.get::<_, rquickjs::Function>("writeFile").is_ok());
            assert!(fs_obj.get::<_, rquickjs::Function>("exists").is_ok());
            assert!(fs_obj.get::<_, rquickjs::Function>("mkdirSync").is_ok());
        });
    }
}
