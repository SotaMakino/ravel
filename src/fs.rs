use rquickjs::{Ctx, Error, Exception, Function, Object, Promise, Result, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::encoding::{bytes_from_value, to_uint8_array};

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

/// Hand a Rust outcome back to JavaScript by calling the promise's own
/// resolve/reject functions. This is the seam between the two worlds: the
/// Tokio task finishes, and settling the promise queues a microtask that the
/// event loop drains.
fn settle<'js>(
    ctx: &Ctx<'js>,
    resolve: Function<'js>,
    reject: Function<'js>,
    outcome: std::result::Result<Value<'js>, String>,
) {
    let called = match outcome {
        Ok(value) => resolve.call::<_, ()>((value,)),
        Err(message) => Exception::from_message(ctx.clone(), &message)
            .and_then(|exception| reject.call::<_, ()>((exception,))),
    };
    if let Err(e) = called {
        eprintln!("Failed to settle promise: {}", e);
    }
}

/// Turn a JS value into the bytes to write. Runs before the write is
/// scheduled, since it needs the JS value.
fn write_payload(path: &str, data: &Value<'_>) -> std::result::Result<Vec<u8>, String> {
    match data.as_string() {
        Some(s) => s
            .to_string()
            .map(String::into_bytes)
            .map_err(|e| format!("{}: '{}'", e, path)),
        None => bytes_from_value(data)
            .map_err(|_| format!("expected a string, Uint8Array, or ArrayBuffer: '{}'", path)),
    }
}

async fn write_bytes(resolved: PathBuf, bytes: Vec<u8>) -> std::result::Result<(), String> {
    if let Some(parent) = resolved.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("failed to create directories: {}: '{}'", e, parent.display()))?;
    }
    tokio::fs::write(&resolved, &bytes)
        .await
        .map_err(|e| format!("{}: '{}'", e, resolved.display()))
}

pub fn setup_fs<'js>(ctx: &Ctx<'js>, root: &Path) -> Result<()> {
    let root1 = root.to_path_buf();
    let root2 = root.to_path_buf();
    let root3 = root.to_path_buf();
    let root4 = root.to_path_buf();
    let root5 = root.to_path_buf();
    let root6 = root.to_path_buf();
    let fs_obj = Object::new(ctx.clone())?;

    // Async. Returns a promise; the read happens on Tokio, off this thread.
    fs_obj.set(
        "readFile",
        rquickjs::function::Func::new(move |ctx: Ctx<'js>, path: String| -> Result<Promise<'js>> {
            let (promise, resolve, reject) = Promise::new(&ctx)?;

            // Check the sandbox now, not on the worker: a path that escapes
            // the root must never reach the filesystem at all.
            let resolved =
                resolve_path(&root1, &path).map_err(|e| format!("{}: '{}'", e, path));

            let task_ctx = ctx.clone();
            ctx.spawn(async move {
                let outcome = match resolved {
                    Ok(resolved) => tokio::fs::read(&resolved)
                        .await
                        .map_err(|e| format!("{}: '{}'", e, resolved.display())),
                    Err(message) => Err(message),
                };
                let outcome = outcome.and_then(|bytes| {
                    to_uint8_array(&task_ctx, bytes).map_err(|e| e.to_string())
                });
                settle(&task_ctx, resolve, reject, outcome);
            });

            Ok(promise)
        }),
    )?;

    fs_obj.set(
        "writeFile",
        rquickjs::function::Func::new(
            move |ctx: Ctx<'js>, path: String, data: Value<'js>| -> Result<Promise<'js>> {
                let (promise, resolve, reject) = Promise::new(&ctx)?;

                let prepared = write_payload(&path, &data).and_then(|bytes| {
                    resolve_path_for_write(&root2, &path)
                        .map_err(|e| format!("{}: '{}'", e, path))
                        .map(|resolved| (resolved, bytes))
                });

                let task_ctx = ctx.clone();
                ctx.spawn(async move {
                    let outcome = match prepared {
                        Ok((resolved, bytes)) => write_bytes(resolved, bytes).await,
                        Err(message) => Err(message),
                    };
                    settle(
                        &task_ctx,
                        resolve,
                        reject,
                        outcome.map(|()| Value::new_undefined(task_ctx.clone())),
                    );
                });

                Ok(promise)
            },
        ),
    )?;

    // Sync equivalents, for build scripts that read top to bottom.
    fs_obj.set(
        "readFileSync",
        rquickjs::function::Func::new(move |ctx: Ctx<'js>, path: String| -> Result<Value<'js>> {
            let resolved = resolve_path(&root5, &path).map_err(|e| {
                Error::new_from_js_message("string", "file", format!("{}: '{}'", e, path))
            })?;
            match fs::read(&resolved) {
                Ok(bytes) => to_uint8_array(&ctx, bytes),
                Err(e) => Err(Error::new_from_js_message(
                    "string",
                    "file",
                    format!("{}: '{}'", e, resolved.display()),
                )),
            }
        }),
    )?;

    fs_obj.set(
        "writeFileSync",
        rquickjs::function::Func::new(move |path: String, data: Value<'js>| -> Result<()> {
            let bytes = write_payload(&path, &data)
                .map_err(|e| Error::new_from_js_message("file", "write", e))?;
            let resolved = resolve_path_for_write(&root6, &path).map_err(|e| {
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
                Error::new_from_js_message("file", "write", format!("{}: '{}'", e, resolved.display()))
            })
        }),
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
            assert!(fs_obj.get::<_, rquickjs::Function>("readFileSync").is_ok());
            assert!(fs_obj.get::<_, rquickjs::Function>("writeFileSync").is_ok());
            assert!(fs_obj.get::<_, rquickjs::Function>("exists").is_ok());
            assert!(fs_obj.get::<_, rquickjs::Function>("mkdirSync").is_ok());
        });
    }
}
