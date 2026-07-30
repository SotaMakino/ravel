//! Turning an import specifier into a file on disk.
//!
//! Three kinds of specifier, three rules:
//!
//!   ./thing   relative -- join it to the importing file's directory
//!   #thing    internal -- look it up in the nearest package.json "imports"
//!   thing     bare     -- walk up the tree looking in node_modules
//!
//! A bare specifier lands on a package directory, and from there `exports`
//! decides what is reachable. That map is the package's public surface: if it
//! exists, nothing outside it can be imported, however real the file is.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

/// Conditions ravel matches in `exports` and `imports`, most specific first.
///
/// `node` is deliberately absent. ravel has no node builtins, so a package's
/// node entry point is the one least likely to run here; letting it fall
/// through to `default` picks the portable build instead. `default` always
/// matches and is handled separately, because it is a fallback rather than a
/// claim about the environment.
const CONDITIONS: &[&str] = &["ravel", "import"];

/// Extensions tried when a specifier has none, in order. `.js` stays first so
/// `./utils` keeps meaning `utils.js` when `utils.ts` sits beside it.
const EXTENSIONS: &[&str] = &["js", "mjs", "ts", "tsx"];

#[derive(Debug, Default, Deserialize)]
struct PackageJson {
    main: Option<String>,
    /// Left as raw JSON: a target can be a string, null, an array of
    /// fallbacks, or an object of conditions, nested to any depth.
    exports: Option<Value>,
    imports: Option<Value>,
}

pub struct ModuleResolver {
    /// Where the entry module lives. Used when the importer has no directory
    /// of its own, which is the case for the entry module itself.
    root: PathBuf,
    /// package.json contents by directory. A deep import chain asks about the
    /// same handful of packages over and over, and none of them change while
    /// the process runs.
    packages: HashMap<PathBuf, Option<Arc<PackageJson>>>,
}

impl ModuleResolver {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            packages: HashMap::new(),
        }
    }

    /// Resolve `specifier` as imported from `base`.
    ///
    /// The path comes back canonical, because the engine keys modules by this
    /// string. `./lib.js` and `./nested/../lib.js` are the same file and must
    /// resolve to the same key, or the module would be evaluated twice and its
    /// state duplicated.
    pub fn resolve(&mut self, base: &str, specifier: &str) -> Result<PathBuf, String> {
        let dir = self.importer_dir(base);

        let found = if specifier.starts_with('#') {
            self.resolve_imports(&dir, specifier)
        } else if specifier.starts_with("./") || specifier.starts_with("../") {
            self.resolve_path(&dir.join(specifier))
        } else if Path::new(specifier).is_absolute() {
            self.resolve_path(Path::new(specifier))
        } else {
            self.resolve_bare(&dir, specifier)
        }?;

        found
            .canonicalize()
            .map_err(|e| format!("cannot read '{}': {}", found.display(), e))
    }

    /// The directory imports are resolved against. The entry module is named
    /// by its file name alone, so it has no parent to speak of and falls back
    /// to the root.
    fn importer_dir(&self, base: &str) -> PathBuf {
        match Path::new(base).parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => self.root.clone(),
        }
    }

    // -- plain paths ------------------------------------------------------

    /// A file, a file with an extension added, or a directory's entry point.
    fn resolve_path(&mut self, path: &Path) -> Result<PathBuf, String> {
        if let Some(file) = as_file(path) {
            return Ok(file);
        }
        if path.is_dir() {
            return self.resolve_directory(path);
        }
        Err(format!("cannot find '{}'", tidy(path).display()))
    }

    /// A directory means its package.json `main`, or failing that its index.
    fn resolve_directory(&mut self, dir: &Path) -> Result<PathBuf, String> {
        if let Some(main) = self.package_json(dir).and_then(|pkg| pkg.main.clone()) {
            let target = dir.join(&main);
            if let Some(file) = as_file(&target).or_else(|| index_of(&target)) {
                return Ok(file);
            }
        }
        index_of(dir).ok_or_else(|| format!("no entry point in '{}'", tidy(dir).display()))
    }

    // -- bare specifiers --------------------------------------------------

    /// Walk up from the importer looking for `node_modules/<package>`.
    ///
    /// Walking up rather than searching one fixed place is what lets two
    /// packages depend on different versions of the same thing: each finds the
    /// copy nearest to itself.
    fn resolve_bare(&mut self, from: &Path, specifier: &str) -> Result<PathBuf, String> {
        let (package, subpath) = split_specifier(specifier)?;

        let mut dir = Some(from);
        while let Some(current) = dir {
            let candidate = current.join("node_modules").join(&package);
            if candidate.is_dir() {
                return self.resolve_package(&candidate, &subpath, specifier);
            }
            dir = current.parent();
        }
        Err(format!(
            "cannot find package '{}' in any node_modules directory above '{}'",
            package,
            from.display()
        ))
    }

    /// Enter a package that was found in node_modules.
    fn resolve_package(
        &mut self,
        pkg_dir: &Path,
        subpath: &str,
        specifier: &str,
    ) -> Result<PathBuf, String> {
        let package = self.package_json(pkg_dir);

        if let Some(exports) = package.as_ref().and_then(|pkg| pkg.exports.clone()) {
            return self.resolve_exports(pkg_dir, &exports, subpath, specifier);
        }

        // No exports map, so the whole directory is fair game -- the old
        // behaviour, and still what most packages rely on.
        if subpath == "." {
            return self.resolve_directory(pkg_dir);
        }
        self.resolve_path(&pkg_dir.join(subpath.trim_start_matches("./")))
    }

    /// Apply a package's `exports` map.
    ///
    /// An exports map is exhaustive. A subpath it does not name is not part of
    /// the package, even if the file is sitting right there.
    fn resolve_exports(
        &mut self,
        pkg_dir: &Path,
        exports: &Value,
        subpath: &str,
        specifier: &str,
    ) -> Result<PathBuf, String> {
        let target = match exports {
            // Sugar: a bare string, or conditions without subpath keys, both
            // describe the package's own entry point and nothing else.
            Value::String(_) | Value::Array(_) => {
                if subpath != "." {
                    return Err(not_exported(specifier, pkg_dir));
                }
                select_target(exports, None)
            }
            Value::Object(map) if !is_subpath_map(map) => {
                if subpath != "." {
                    return Err(not_exported(specifier, pkg_dir));
                }
                select_target(exports, None)
            }
            Value::Object(map) => match match_subpath(map, subpath) {
                Some((entry, wildcard)) => select_target(entry, wildcard.as_deref()),
                None => return Err(not_exported(specifier, pkg_dir)),
            },
            _ => None,
        };

        let target = target.ok_or_else(|| not_exported(specifier, pkg_dir))?;
        let path = self.target_path(pkg_dir, &target, specifier)?;
        as_file(&path).ok_or_else(|| {
            format!(
                "'{}' points at '{}', which does not exist",
                specifier,
                tidy(&path).display()
            )
        })
    }

    // -- internal (#) specifiers -----------------------------------------

    /// Resolve a `#name` specifier against the nearest enclosing package.
    fn resolve_imports(&mut self, from: &Path, specifier: &str) -> Result<PathBuf, String> {
        let Some(pkg_dir) = self.nearest_package_dir(from) else {
            return Err(format!(
                "'{}' needs a package.json with an \"imports\" map, and there is none above '{}'",
                specifier,
                from.display()
            ));
        };

        let imports = self
            .package_json(&pkg_dir)
            .and_then(|pkg| pkg.imports.clone());
        let Some(Value::Object(map)) = imports else {
            return Err(format!(
                "'{}' is not listed in the \"imports\" of '{}'",
                specifier,
                pkg_dir.display()
            ));
        };

        let Some((entry, wildcard)) = match_subpath(&map, specifier) else {
            return Err(format!(
                "'{}' is not listed in the \"imports\" of '{}'",
                specifier,
                pkg_dir.display()
            ));
        };
        let Some(target) = select_target(entry, wildcard.as_deref()) else {
            return Err(format!(
                "'{}' has no target matching this runtime's conditions",
                specifier
            ));
        };

        // Unlike exports, an imports target may be a bare specifier: mapping
        // "#dep" onto a real dependency is the point of the feature.
        if target.starts_with("./") || target.starts_with("../") {
            let path = self.target_path(&pkg_dir, &target, specifier)?;
            return as_file(&path).ok_or_else(|| {
                format!(
                    "'{}' points at '{}', which does not exist",
                    specifier,
                    tidy(&path).display()
                )
            });
        }
        self.resolve_bare(&pkg_dir, &target)
    }

    // -- helpers ----------------------------------------------------------

    /// Join a target onto its package, refusing anything that climbs out.
    ///
    /// Without this, `"./*": "./../../*"` would turn a package's own exports
    /// map into a way to read the rest of the disk.
    fn target_path(
        &self,
        pkg_dir: &Path,
        target: &str,
        specifier: &str,
    ) -> Result<PathBuf, String> {
        if !target.starts_with("./") && !target.starts_with("../") {
            return Err(format!(
                "'{}' maps to '{}', which is not a relative path",
                specifier, target
            ));
        }
        let joined = pkg_dir.join(target);
        if !stays_within(pkg_dir, &joined) {
            return Err(format!(
                "'{}' maps to '{}', which escapes its package",
                specifier, target
            ));
        }
        Ok(joined)
    }

    /// Nearest directory at or above `from` holding a package.json.
    fn nearest_package_dir(&mut self, from: &Path) -> Option<PathBuf> {
        let mut dir = Some(from);
        while let Some(current) = dir {
            if current.join("package.json").is_file() {
                return Some(current.to_path_buf());
            }
            dir = current.parent();
        }
        None
    }

    /// Read and cache a directory's package.json. A missing or unreadable one
    /// is cached too, so a bad path is not re-read on every import.
    fn package_json(&mut self, dir: &Path) -> Option<Arc<PackageJson>> {
        if let Some(cached) = self.packages.get(dir) {
            return cached.clone();
        }
        let parsed = std::fs::read_to_string(dir.join("package.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<PackageJson>(&text).ok())
            .map(Arc::new);
        self.packages.insert(dir.to_path_buf(), parsed.clone());
        parsed
    }
}

/// The file at `path`, or the file found by adding a known extension.
fn as_file(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    EXTENSIONS.iter().find_map(|ext| {
        let mut candidate = path.as_os_str().to_os_string();
        candidate.push(".");
        candidate.push(ext);
        let candidate = PathBuf::from(candidate);
        candidate.is_file().then_some(candidate)
    })
}

fn index_of(dir: &Path) -> Option<PathBuf> {
    EXTENSIONS.iter().find_map(|ext| {
        let candidate = dir.join(format!("index.{}", ext));
        candidate.is_file().then_some(candidate)
    })
}

/// Split `@scope/pkg/sub` into the package name and the subpath below it.
fn split_specifier(specifier: &str) -> Result<(String, String), String> {
    if specifier.is_empty() {
        return Err("empty import specifier".to_string());
    }
    let mut parts = specifier.splitn(2, '/');
    let first = parts.next().unwrap_or_default();

    // A scoped name spans two segments, so take one more before the subpath.
    let (name, rest) = if first.starts_with('@') {
        let remainder = parts.next().unwrap_or_default();
        if remainder.is_empty() {
            return Err(format!("'{}' is missing a package name", specifier));
        }
        let mut scoped = remainder.splitn(2, '/');
        let second = scoped.next().unwrap_or_default();
        (format!("{}/{}", first, second), scoped.next())
    } else {
        (first.to_string(), parts.next())
    };

    let subpath = match rest {
        Some(rest) if !rest.is_empty() => format!("./{}", rest),
        _ => ".".to_string(),
    };
    Ok((name, subpath))
}

/// An exports object is a subpath map when its keys are subpaths. Mixing the
/// two forms is invalid, so the first key settles it.
fn is_subpath_map(map: &serde_json::Map<String, Value>) -> bool {
    map.keys().next().is_some_and(|key| key.starts_with('.'))
}

/// Find the map entry for `subpath`: an exact key, or the pattern key with the
/// longest matching prefix. Returns the entry and whatever `*` stood for.
fn match_subpath<'a>(
    map: &'a serde_json::Map<String, Value>,
    subpath: &str,
) -> Option<(&'a Value, Option<String>)> {
    if let Some(entry) = map.get(subpath) {
        return Some((entry, None));
    }

    let mut best: Option<(usize, &Value, String)> = None;
    for (key, entry) in map {
        let Some((prefix, suffix)) = key.split_once('*') else {
            continue;
        };
        // A second star would make the match ambiguous, so the key is invalid.
        if suffix.contains('*') {
            continue;
        }
        if !subpath.starts_with(prefix) || !subpath.ends_with(suffix) {
            continue;
        }
        if subpath.len() < prefix.len() + suffix.len() {
            continue;
        }
        let wildcard = subpath[prefix.len()..subpath.len() - suffix.len()].to_string();
        // The most specific pattern wins, which is the one that fixed the most
        // characters before the star.
        if best.as_ref().is_none_or(|(len, _, _)| prefix.len() > *len) {
            best = Some((prefix.len(), entry, wildcard));
        }
    }
    best.map(|(_, entry, wildcard)| (entry, Some(wildcard)))
}

/// Walk a target down to a single path, choosing between conditions.
///
/// Conditions are tried in the order the package.json lists them, which is why
/// `"default"` belongs last: whoever writes the map decides the priority, not
/// us. `null` means the target is deliberately unavailable.
fn select_target(target: &Value, wildcard: Option<&str>) -> Option<String> {
    match target {
        Value::String(literal) => Some(match wildcard {
            Some(value) => literal.replace('*', value),
            None => literal.clone(),
        }),
        Value::Object(conditions) => conditions
            .iter()
            .filter(|(key, _)| key.as_str() == "default" || CONDITIONS.contains(&key.as_str()))
            .find_map(|(_, entry)| select_target(entry, wildcard)),
        // A list of fallbacks: the first one we understand wins.
        Value::Array(entries) => entries
            .iter()
            .find_map(|entry| select_target(entry, wildcard)),
        _ => None,
    }
}

/// Fold away `.` and `..` without touching the disk. Used for messages and for
/// containment checks, where the path may not exist and canonicalize would
/// fail. Returns `None` if the path climbs above its own root.
fn normalize(path: &Path) -> Option<PathBuf> {
    let mut depth: i32 = 0;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                out.pop();
            }
            other => {
                depth += 1;
                out.push(other);
            }
        }
    }
    Some(out)
}

/// A path fit to show a person: no `./` or `..` left in the middle of it.
fn tidy(path: &Path) -> PathBuf {
    normalize(path).unwrap_or_else(|| path.to_path_buf())
}

/// Whether `path` stays inside `base` once `..` segments are folded in.
fn stays_within(base: &Path, path: &Path) -> bool {
    normalize(path).is_some_and(|normalized| normalized.starts_with(base))
}

fn not_exported(specifier: &str, pkg_dir: &Path) -> String {
    format!("'{}' is not exported by '{}'", specifier, pkg_dir.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build a tree of files from `(relative path, contents)` pairs.
    fn tree(files: &[(&str, &str)]) -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, contents) in files {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, contents).unwrap();
        }
        dir
    }

    fn resolver(dir: &TempDir) -> ModuleResolver {
        ModuleResolver::new(&dir.path().canonicalize().unwrap())
    }

    /// Resolve `specifier` as imported by `<root>/main.js`.
    fn resolve(dir: &TempDir, specifier: &str) -> Result<PathBuf, String> {
        let root = dir.path().canonicalize().unwrap();
        let base = root.join("main.js");
        resolver(dir).resolve(base.to_str().unwrap(), specifier)
    }

    fn assert_resolves(dir: &TempDir, specifier: &str, expected: &str) {
        let got = resolve(dir, specifier).unwrap_or_else(|e| panic!("{}: {}", specifier, e));
        let want = dir.path().canonicalize().unwrap().join(expected);
        assert_eq!(got, want, "resolving {}", specifier);
    }

    // -- relative paths ---------------------------------------------------

    #[test]
    fn test_relative_import_with_extension() {
        let dir = tree(&[("lib.js", "")]);
        assert_resolves(&dir, "./lib.js", "lib.js");
    }

    #[test]
    fn test_relative_import_without_extension() {
        let dir = tree(&[("lib.js", "")]);
        assert_resolves(&dir, "./lib", "lib.js");
    }

    #[test]
    fn test_extensionless_import_finds_typescript() {
        let dir = tree(&[("lib.ts", "")]);
        assert_resolves(&dir, "./lib", "lib.ts");
    }

    #[test]
    fn test_javascript_wins_when_both_extensions_exist() {
        let dir = tree(&[("lib.js", ""), ("lib.ts", "")]);
        assert_resolves(&dir, "./lib", "lib.js");
    }

    #[test]
    fn test_parent_relative_import() {
        let dir = tree(&[("shared.js", ""), ("nested/main.js", "")]);
        let root = dir.path().canonicalize().unwrap();
        let base = root.join("nested/main.js");
        let got = resolver(&dir)
            .resolve(base.to_str().unwrap(), "../shared.js")
            .unwrap();
        assert_eq!(got, root.join("shared.js"));
    }

    #[test]
    fn test_directory_import_uses_index() {
        let dir = tree(&[("utils/index.js", "")]);
        assert_resolves(&dir, "./utils", "utils/index.js");
    }

    #[test]
    fn test_directory_import_uses_package_main() {
        let dir = tree(&[
            ("utils/package.json", r#"{"main": "./entry.js"}"#),
            ("utils/entry.js", ""),
            ("utils/index.js", ""),
        ]);
        assert_resolves(&dir, "./utils", "utils/entry.js");
    }

    #[test]
    fn test_two_routes_to_one_file_give_one_key() {
        // The engine keys modules by the resolved path, so a file reached two
        // ways has to come back identical or it would be evaluated twice.
        let dir = tree(&[("lib.js", ""), ("nested/placeholder.js", "")]);
        let direct = resolve(&dir, "./lib.js").unwrap();
        let roundabout = resolve(&dir, "./nested/../lib.js").unwrap();
        assert_eq!(direct, roundabout);
    }

    #[test]
    fn test_missing_relative_import_is_an_error() {
        let dir = tree(&[("main.js", "")]);
        assert!(resolve(&dir, "./nope.js").is_err());
    }

    #[test]
    fn test_dotted_filename_keeps_its_own_extension() {
        // Adding an extension must not overwrite one that is already there.
        let dir = tree(&[("data.min.js", "")]);
        assert_resolves(&dir, "./data.min.js", "data.min.js");
    }

    // -- node_modules lookup ----------------------------------------------

    #[test]
    fn test_bare_import_finds_node_modules() {
        let dir = tree(&[
            ("node_modules/left-pad/package.json", r#"{"main": "i.js"}"#),
            ("node_modules/left-pad/i.js", ""),
        ]);
        assert_resolves(&dir, "left-pad", "node_modules/left-pad/i.js");
    }

    #[test]
    fn test_bare_import_falls_back_to_index() {
        let dir = tree(&[("node_modules/tiny/index.js", "")]);
        assert_resolves(&dir, "tiny", "node_modules/tiny/index.js");
    }

    #[test]
    fn test_bare_import_walks_up_to_a_parent_node_modules() {
        let dir = tree(&[("node_modules/dep/index.js", ""), ("a/b/c/main.js", "")]);
        let root = dir.path().canonicalize().unwrap();
        let base = root.join("a/b/c/main.js");
        let got = resolver(&dir)
            .resolve(base.to_str().unwrap(), "dep")
            .unwrap();
        assert_eq!(got, root.join("node_modules/dep/index.js"));
    }

    #[test]
    fn test_nearest_node_modules_wins() {
        // Two copies of one package. The importer gets the closer one, which
        // is how two versions coexist in one tree.
        let dir = tree(&[
            ("node_modules/dep/index.js", "outer"),
            ("app/node_modules/dep/index.js", "inner"),
            ("app/main.js", ""),
        ]);
        let root = dir.path().canonicalize().unwrap();
        let base = root.join("app/main.js");
        let got = resolver(&dir)
            .resolve(base.to_str().unwrap(), "dep")
            .unwrap();
        assert_eq!(got, root.join("app/node_modules/dep/index.js"));
    }

    #[test]
    fn test_scoped_package() {
        let dir = tree(&[("node_modules/@acme/tools/index.js", "")]);
        assert_resolves(&dir, "@acme/tools", "node_modules/@acme/tools/index.js");
    }

    #[test]
    fn test_scoped_package_subpath() {
        let dir = tree(&[("node_modules/@acme/tools/fp.js", "")]);
        assert_resolves(&dir, "@acme/tools/fp", "node_modules/@acme/tools/fp.js");
    }

    #[test]
    fn test_deep_subpath_without_exports() {
        let dir = tree(&[("node_modules/dep/lib/deep.js", "")]);
        assert_resolves(&dir, "dep/lib/deep.js", "node_modules/dep/lib/deep.js");
    }

    #[test]
    fn test_missing_package_is_an_error() {
        let dir = tree(&[("main.js", "")]);
        let err = resolve(&dir, "ghost").unwrap_err();
        assert!(err.contains("cannot find package 'ghost'"), "got: {}", err);
    }

    #[test]
    fn test_scoped_name_without_a_package_is_an_error() {
        let dir = tree(&[("main.js", "")]);
        assert!(resolve(&dir, "@acme").is_err());
    }

    // -- exports ----------------------------------------------------------

    #[test]
    fn test_exports_string_shorthand() {
        let dir = tree(&[
            ("node_modules/dep/package.json", r#"{"exports": "./m.js"}"#),
            ("node_modules/dep/m.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/m.js");
    }

    #[test]
    fn test_exports_subpath_map() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {".": "./main.js", "./extra": "./x.js"}}"#,
            ),
            ("node_modules/dep/main.js", ""),
            ("node_modules/dep/x.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/main.js");
        assert_resolves(&dir, "dep/extra", "node_modules/dep/x.js");
    }

    #[test]
    fn test_exports_hides_files_it_does_not_name() {
        // The file exists, but the package did not publish it.
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {".": "./main.js"}}"#,
            ),
            ("node_modules/dep/main.js", ""),
            ("node_modules/dep/private.js", ""),
        ]);
        let err = resolve(&dir, "dep/private.js").unwrap_err();
        assert!(err.contains("is not exported"), "got: {}", err);
    }

    #[test]
    fn test_exports_null_blocks_a_subpath() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {".": "./main.js", "./internal": null}}"#,
            ),
            ("node_modules/dep/main.js", ""),
            ("node_modules/dep/internal.js", ""),
        ]);
        assert!(resolve(&dir, "dep/internal").is_err());
    }

    #[test]
    fn test_exports_wildcard_pattern() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"./features/*": "./src/features/*.js"}}"#,
            ),
            ("node_modules/dep/src/features/one.js", ""),
        ]);
        assert_resolves(
            &dir,
            "dep/features/one",
            "node_modules/dep/src/features/one.js",
        );
    }

    #[test]
    fn test_exports_most_specific_pattern_wins() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"./*": "./generic/*.js", "./deep/*": "./special/*.js"}}"#,
            ),
            ("node_modules/dep/generic/deep/thing.js", ""),
            ("node_modules/dep/special/thing.js", ""),
        ]);
        assert_resolves(&dir, "dep/deep/thing", "node_modules/dep/special/thing.js");
    }

    #[test]
    fn test_exports_pattern_cannot_escape_the_package() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"./*": "./*.js"}}"#,
            ),
            ("secret.js", ""),
        ]);
        let err = resolve(&dir, "dep/../../secret").unwrap_err();
        assert!(err.contains("escapes its package"), "got: {}", err);
    }

    #[test]
    fn test_exports_target_must_be_relative() {
        let dir = tree(&[(
            "node_modules/dep/package.json",
            r#"{"exports": {".": "/etc/passwd"}}"#,
        )]);
        let err = resolve(&dir, "dep").unwrap_err();
        assert!(err.contains("not a relative path"), "got: {}", err);
    }

    #[test]
    fn test_exports_pointing_at_a_missing_file_is_an_error() {
        let dir = tree(&[(
            "node_modules/dep/package.json",
            r#"{"exports": {".": "./gone.js"}}"#,
        )]);
        let err = resolve(&dir, "dep").unwrap_err();
        assert!(err.contains("does not exist"), "got: {}", err);
    }

    #[test]
    fn test_exports_beats_main() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"main": "./old.js", "exports": "./new.js"}"#,
            ),
            ("node_modules/dep/old.js", ""),
            ("node_modules/dep/new.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/new.js");
    }

    // -- conditional exports ----------------------------------------------

    #[test]
    fn test_conditions_prefer_import_over_require() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"require": "./cjs.js", "import": "./esm.js"}}"#,
            ),
            ("node_modules/dep/cjs.js", ""),
            ("node_modules/dep/esm.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/esm.js");
    }

    #[test]
    fn test_conditions_fall_through_to_default() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"browser": "./b.js", "default": "./d.js"}}"#,
            ),
            ("node_modules/dep/b.js", ""),
            ("node_modules/dep/d.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/d.js");
    }

    #[test]
    fn test_ravel_condition_wins_when_offered() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"ravel": "./r.js", "import": "./esm.js", "default": "./d.js"}}"#,
            ),
            ("node_modules/dep/r.js", ""),
            ("node_modules/dep/esm.js", ""),
            ("node_modules/dep/d.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/r.js");
    }

    #[test]
    fn test_conditions_are_matched_in_declaration_order() {
        // "default" is listed first here, so it wins even though "import"
        // would also match. Priority belongs to the package, not to us.
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"default": "./d.js", "import": "./esm.js"}}"#,
            ),
            ("node_modules/dep/d.js", ""),
            ("node_modules/dep/esm.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/d.js");
    }

    #[test]
    fn test_conditions_nest_inside_subpaths() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {"./sub": {"import": "./sub-esm.js", "default": "./sub.js"}}}"#,
            ),
            ("node_modules/dep/sub-esm.js", ""),
            ("node_modules/dep/sub.js", ""),
        ]);
        assert_resolves(&dir, "dep/sub", "node_modules/dep/sub-esm.js");
    }

    #[test]
    fn test_unmatched_conditions_are_an_error() {
        let dir = tree(&[(
            "node_modules/dep/package.json",
            r#"{"exports": {"browser": "./b.js"}}"#,
        )]);
        assert!(resolve(&dir, "dep").is_err());
    }

    #[test]
    fn test_array_target_takes_the_first_understood_entry() {
        let dir = tree(&[
            (
                "node_modules/dep/package.json",
                r#"{"exports": {".": [{"browser": "./b.js"}, "./fallback.js"]}}"#,
            ),
            ("node_modules/dep/fallback.js", ""),
        ]);
        assert_resolves(&dir, "dep", "node_modules/dep/fallback.js");
    }

    // -- imports ----------------------------------------------------------

    #[test]
    fn test_imports_maps_a_hash_specifier_to_a_file() {
        let dir = tree(&[
            ("package.json", r##"{"imports": {"#config": "./cfg.js"}}"##),
            ("cfg.js", ""),
        ]);
        assert_resolves(&dir, "#config", "cfg.js");
    }

    #[test]
    fn test_imports_honours_conditions() {
        let dir = tree(&[
            (
                "package.json",
                r##"{"imports": {"#env": {"import": "./esm.js", "default": "./d.js"}}}"##,
            ),
            ("esm.js", ""),
            ("d.js", ""),
        ]);
        assert_resolves(&dir, "#env", "esm.js");
    }

    #[test]
    fn test_imports_pattern() {
        let dir = tree(&[
            ("package.json", r##"{"imports": {"#lib/*": "./src/*.js"}}"##),
            ("src/thing.js", ""),
        ]);
        assert_resolves(&dir, "#lib/thing", "src/thing.js");
    }

    #[test]
    fn test_imports_can_point_at_a_package() {
        let dir = tree(&[
            ("package.json", r##"{"imports": {"#dep": "real-dep"}}"##),
            ("node_modules/real-dep/index.js", ""),
        ]);
        assert_resolves(&dir, "#dep", "node_modules/real-dep/index.js");
    }

    #[test]
    fn test_imports_resolves_from_the_nearest_package() {
        let dir = tree(&[
            ("package.json", r##"{"imports": {"#x": "./outer.js"}}"##),
            ("outer.js", ""),
            ("app/package.json", r##"{"imports": {"#x": "./inner.js"}}"##),
            ("app/inner.js", ""),
            ("app/main.js", ""),
        ]);
        let root = dir.path().canonicalize().unwrap();
        let base = root.join("app/main.js");
        let got = resolver(&dir)
            .resolve(base.to_str().unwrap(), "#x")
            .unwrap();
        assert_eq!(got, root.join("app/inner.js"));
    }

    #[test]
    fn test_unlisted_hash_specifier_is_an_error() {
        let dir = tree(&[("package.json", r##"{"imports": {"#a": "./a.js"}}"##)]);
        let err = resolve(&dir, "#b").unwrap_err();
        assert!(err.contains("not listed"), "got: {}", err);
    }

    #[test]
    fn test_hash_specifier_without_a_package_json_is_an_error() {
        let dir = tree(&[("main.js", "")]);
        assert!(resolve(&dir, "#anything").is_err());
    }

    // -- unit-level helpers ------------------------------------------------

    #[test]
    fn test_split_specifier_forms() {
        assert_eq!(
            split_specifier("dep").unwrap(),
            ("dep".to_string(), ".".to_string())
        );
        assert_eq!(
            split_specifier("dep/sub/deep").unwrap(),
            ("dep".to_string(), "./sub/deep".to_string())
        );
        assert_eq!(
            split_specifier("@s/p").unwrap(),
            ("@s/p".to_string(), ".".to_string())
        );
        assert_eq!(
            split_specifier("@s/p/sub").unwrap(),
            ("@s/p".to_string(), "./sub".to_string())
        );
        assert!(split_specifier("").is_err());
        assert!(split_specifier("@s").is_err());
    }

    #[test]
    fn test_stays_within() {
        let base = Path::new("/pkg");
        assert!(stays_within(base, Path::new("/pkg/lib/a.js")));
        assert!(stays_within(base, Path::new("/pkg/lib/../a.js")));
        assert!(!stays_within(base, Path::new("/pkg/../other/a.js")));
        assert!(!stays_within(base, Path::new("/elsewhere/a.js")));
    }

    #[test]
    fn test_select_target_skips_unknown_conditions() {
        let target: Value =
            serde_json::from_str(r#"{"deno": "./d.js", "import": "./i.js"}"#).unwrap();
        assert_eq!(select_target(&target, None).as_deref(), Some("./i.js"));
    }

    #[test]
    fn test_select_target_substitutes_every_star() {
        let target = Value::String("./src/*/index-*.js".to_string());
        assert_eq!(
            select_target(&target, Some("thing")).as_deref(),
            Some("./src/thing/index-thing.js")
        );
    }

    #[test]
    fn test_select_target_rejects_null() {
        assert_eq!(select_target(&Value::Null, None), None);
    }

    #[test]
    fn test_package_json_is_cached() {
        let dir = tree(&[("node_modules/dep/index.js", "")]);
        let root = dir.path().canonicalize().unwrap();
        let mut resolver = ModuleResolver::new(&root);
        let pkg_dir = root.join("node_modules/dep");
        assert!(resolver.package_json(&pkg_dir).is_none());
        // Written after the first miss: the cached "no package.json" answer
        // must survive the file appearing later in the same process.
        fs::write(pkg_dir.join("package.json"), r#"{"main": "index.js"}"#).unwrap();
        assert!(resolver.package_json(&pkg_dir).is_none());
    }
}
