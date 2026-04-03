use rquickjs::{Ctx, Error, Object, Result, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn resolve_path(root: &Path, input: &str) -> std::io::Result<PathBuf> {
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
    let canon_root = root.canonicalize()?;

    let joined = root.join(input);

    let canon_resolved = if joined.exists() {
        joined.canonicalize()?
    } else {
        let parent = joined.parent().unwrap_or(Path::new(""));
        let canon_parent = parent.canonicalize()?;

        if !canon_parent.starts_with(&canon_root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "permission denied",
            ));
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
            move |path: String, data: rquickjs::TypedArray<u8>| -> Result<()> {
                let resolved = resolve_path_for_write(&root2, &path).map_err(|e| {
                    Error::new_from_js_message("file", "write", format!("{}: '{}'", e, path))
                })?;
                fs::write(&resolved, data.as_bytes().unwrap()).map_err(|e| {
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
        });
    }
}
