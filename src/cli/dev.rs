//! The dev server: the project served to a browser, unbundled.
//!
//! A browser cannot resolve `import "preact"` -- bare specifiers are not a
//! thing it knows about. Two pieces close that gap:
//!
//!   * an import map, generated from node_modules and injected into the HTML,
//!     which tells the browser that "preact" means a URL;
//!   * `/@id/<specifier>`, which runs the same resolver ravel uses and
//!     redirects to the real file.
//!
//! The redirect is what makes it correct rather than merely convenient. The
//! browser ends up with the module's true path as its URL, so a relative
//! import inside a package resolves against its actual neighbours. Serving the
//! file under the specifier's URL instead would put `./util.js` in the wrong
//! directory. Import maps apply at every depth, so a bare import *inside* a
//! package is resolved by the same map without any rewriting of source.

use std::path::{Path, PathBuf};

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::core::resolver::{BROWSER_CONDITIONS, ModuleResolver};
use crate::transpiler::{is_typescript_file, transpile_ts};

/// Prefix for "resolve this bare specifier and tell me where it went".
const ID_PREFIX: &str = "/@id/";

/// Extensions probed when a request has none, matching what the resolver does
/// for imports so `import "./util"` works the same in a browser.
const EXTENSIONS: &[&str] = &["js", "mjs", "ts", "tsx"];

/// Keep a request inside the project. A browser will happily ask for
/// `../../.ssh/id_rsa` if something puts that in a URL.
fn safe_join(root: &Path, request: &str) -> Option<PathBuf> {
    let relative = request.trim_start_matches('/');
    if relative.contains('\0') {
        return None;
    }
    let joined = root.join(relative);
    // Compare lexically: the path may legitimately not exist yet.
    let mut depth: i32 = 0;
    for component in joined.components() {
        match component {
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            _ => depth += 1,
        }
    }
    joined.canonicalize().ok().filter(|p| p.starts_with(root))
}

/// The file a request names: exact, or with an extension added.
fn find_file(root: &Path, request: &str) -> Option<PathBuf> {
    // Not `?`: an extensionless request names a path that does not exist as
    // written, so giving up here would make the probe below unreachable.
    if let Some(path) = safe_join(root, request)
        && path.is_file()
    {
        return Some(path);
    }
    EXTENSIONS
        .iter()
        .find_map(|ext| safe_join(root, &format!("{}.{}", request, ext)).filter(|p| p.is_file()))
}

/// A URL a browser can ask for, for a file inside the project.
fn url_for(root: &Path, file: &Path) -> Option<String> {
    let relative = file.strip_prefix(root).ok()?;
    let mut url = String::from("/");
    for component in relative.components() {
        if url.len() > 1 {
            url.push('/');
        }
        url.push_str(&component.as_os_str().to_string_lossy());
    }
    Some(url)
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        // Transpiled before it is sent, so it leaves as JavaScript.
        Some("js" | "mjs" | "ts" | "tsx") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// Every package directly inside node_modules, scoped names included.
fn installed_packages(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = std::fs::read_dir(root.join("node_modules")) else {
        return names;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if let Some(scope) = name.strip_prefix('@') {
            // A scope is a directory of packages, not a package.
            let Ok(scoped) = std::fs::read_dir(entry.path()) else {
                continue;
            };
            for inner in scoped.flatten() {
                if inner.path().is_dir() {
                    names.push(format!(
                        "@{}/{}",
                        scope,
                        inner.file_name().to_string_lossy()
                    ));
                }
            }
        } else {
            names.push(name);
        }
    }
    names.sort();
    names
}

/// The import map handed to the browser.
///
/// Two entries per package: the name itself, and the name with a trailing
/// slash so subpaths route through the resolver too. Both point at `/@id/`
/// rather than at a file, because only the resolver knows what a package's
/// exports map makes of a given subpath.
pub fn import_map(root: &Path) -> String {
    let packages = installed_packages(root);
    let mut entries = Vec::new();
    for name in &packages {
        entries.push(format!(r#"    "{0}": "{1}{0}""#, name, ID_PREFIX));
        entries.push(format!(r#"    "{0}/": "{1}{0}/""#, name, ID_PREFIX));
    }
    format!("{{\n  \"imports\": {{\n{}\n  }}\n}}", entries.join(",\n"))
}

/// Put the import map in the document, before anything that might import.
fn inject_import_map(html: &str, map: &str) -> String {
    let tag = format!("<script type=\"importmap\">\n{}\n</script>\n", map);
    match html.find("</head>") {
        Some(at) => format!("{}{}{}", &html[..at], tag, &html[at..]),
        // No head to speak of, so lead with it: an import map has to be
        // parsed before the first module script that relies on it.
        None => format!("{}{}", tag, html),
    }
}

fn serve_file(root: &Path, path: &Path) -> Response {
    let Ok(bytes) = std::fs::read(path) else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let content_type = content_type_for(path);

    if is_typescript_file(&path.to_string_lossy()) {
        let source = String::from_utf8_lossy(&bytes).to_string();
        return match transpile_ts(&source, &path.to_string_lossy()) {
            Ok(js) => ([(header::CONTENT_TYPE, content_type)], js).into_response(),
            Err(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("transpile failed: {}", message),
            )
                .into_response(),
        };
    }

    if path.extension().and_then(|e| e.to_str()) == Some("html") {
        let html = String::from_utf8_lossy(&bytes).to_string();
        let injected = inject_import_map(&html, &import_map(root));
        return ([(header::CONTENT_TYPE, content_type)], injected).into_response();
    }

    ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
}

/// Resolve a bare specifier and send the browser to where it actually lives.
fn redirect_to_module(root: &Path, specifier: &str) -> Response {
    let mut resolver = ModuleResolver::with_conditions(root, BROWSER_CONDITIONS);
    // The importer is the project root, which is where a browser's bare
    // specifier is asking from.
    let base = root.join("index.html");
    match resolver.resolve(&base.to_string_lossy(), specifier) {
        Ok(file) => match url_for(root, &file) {
            Some(url) => (StatusCode::FOUND, [(header::LOCATION, url)]).into_response(),
            // Reachable by walking up out of the project, which a browser has
            // no URL for.
            None => (
                StatusCode::NOT_FOUND,
                format!(
                    "'{}' resolved to {}, which is outside the project",
                    specifier,
                    file.display()
                ),
            )
                .into_response(),
        },
        Err(message) => (
            StatusCode::NOT_FOUND,
            format!("cannot resolve '{}': {}", specifier, message),
        )
            .into_response(),
    }
}

pub fn handle(root: &Path, request_path: &str) -> Response {
    if let Some(specifier) = request_path.strip_prefix(ID_PREFIX) {
        if specifier.is_empty() {
            return (StatusCode::NOT_FOUND, "no specifier").into_response();
        }
        return redirect_to_module(root, specifier);
    }

    let request = if request_path == "/" {
        "index.html"
    } else {
        request_path
    };

    match find_file(root, request) {
        Some(file) => serve_file(root, &file),
        None => (
            StatusCode::NOT_FOUND,
            format!("not found: {}", request_path),
        )
            .into_response(),
    }
}

pub fn dev(port: u16) {
    let root = std::env::current_dir().expect("Failed to read the working directory");
    let root = root.canonicalize().unwrap_or(root);

    if !root.join("index.html").is_file() {
        eprintln!("Error: no index.html in {}", root.display());
        eprintln!("The dev server serves the project as-is, starting from index.html.");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let serve_root = root.clone();
        let app = axum::Router::new().fallback(move |req: axum::extract::Request| {
            let root = serve_root.clone();
            async move {
                let path = req.uri().path().to_string();
                handle(&root, &path)
            }
        });

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let packages = installed_packages(&root).len();
        println!("Serving {} at http://{}", root.display(), addr);
        println!(
            "{} package{} in the import map",
            packages,
            if packages == 1 { "" } else { "s" }
        );

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");
        axum::serve(listener, app).await.expect("Server error");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tree(files: &[(&str, &str)]) -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, contents) in files {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, contents).unwrap();
        }
        dir
    }

    fn root_of(dir: &TempDir) -> PathBuf {
        dir.path().canonicalize().unwrap()
    }

    #[test]
    fn test_safe_join_stays_inside_the_project() {
        let dir = tree(&[("a.js", "")]);
        let root = root_of(&dir);
        assert!(safe_join(&root, "/a.js").is_some());
        assert!(safe_join(&root, "/../a.js").is_none());
        assert!(safe_join(&root, "/../../etc/passwd").is_none());
        assert!(safe_join(&root, "/a\0.js").is_none());
    }

    #[test]
    fn test_find_file_adds_a_missing_extension() {
        let dir = tree(&[("util.js", ""), ("typed.ts", "")]);
        let root = root_of(&dir);
        assert_eq!(find_file(&root, "/util").unwrap(), root.join("util.js"));
        assert_eq!(find_file(&root, "/typed").unwrap(), root.join("typed.ts"));
        assert!(find_file(&root, "/gone").is_none());
    }

    #[test]
    fn test_url_for_is_relative_to_the_project() {
        let dir = tree(&[("node_modules/dep/i.js", "")]);
        let root = root_of(&dir);
        assert_eq!(
            url_for(&root, &root.join("node_modules/dep/i.js")).unwrap(),
            "/node_modules/dep/i.js"
        );
    }

    #[test]
    fn test_url_for_refuses_paths_outside_the_project() {
        let dir = tree(&[("a.js", "")]);
        assert!(url_for(&root_of(&dir), Path::new("/etc/passwd")).is_none());
    }

    #[test]
    fn test_installed_packages_lists_plain_and_scoped() {
        let dir = tree(&[
            ("node_modules/preact/package.json", "{}"),
            ("node_modules/@acme/tools/package.json", "{}"),
            ("node_modules/.bin/whatever", ""),
        ]);
        let found = installed_packages(&root_of(&dir));
        assert_eq!(found, vec!["@acme/tools".to_string(), "preact".to_string()]);
    }

    #[test]
    fn test_installed_packages_without_node_modules() {
        let dir = tree(&[("index.html", "")]);
        assert!(installed_packages(&root_of(&dir)).is_empty());
    }

    #[test]
    fn test_import_map_has_a_bare_and_a_prefix_entry() {
        let dir = tree(&[("node_modules/preact/package.json", "{}")]);
        let map = import_map(&root_of(&dir));
        assert!(map.contains(r#""preact": "/@id/preact""#), "map: {}", map);
        assert!(map.contains(r#""preact/": "/@id/preact/""#), "map: {}", map);
    }

    #[test]
    fn test_import_map_is_valid_json() {
        let dir = tree(&[
            ("node_modules/preact/package.json", "{}"),
            ("node_modules/@acme/tools/package.json", "{}"),
        ]);
        let map = import_map(&root_of(&dir));
        let parsed: serde_json::Value = serde_json::from_str(&map).expect("not JSON");
        assert!(parsed["imports"]["preact"].is_string());
        assert!(parsed["imports"]["@acme/tools"].is_string());
    }

    #[test]
    fn test_import_map_with_no_packages_is_still_valid_json() {
        let dir = tree(&[("index.html", "")]);
        let map = import_map(&root_of(&dir));
        serde_json::from_str::<serde_json::Value>(&map).expect("not JSON");
    }

    #[test]
    fn test_inject_import_map_goes_before_head_closes() {
        let html = "<html><head><title>t</title></head><body></body></html>";
        let out = inject_import_map(html, "{}");
        let map_at = out.find("importmap").unwrap();
        let head_end = out.find("</head>").unwrap();
        let body_at = out.find("<body>").unwrap();
        assert!(map_at < head_end, "map landed outside head: {}", out);
        assert!(map_at < body_at);
    }

    #[test]
    fn test_inject_import_map_without_a_head() {
        // Still has to come first, or a module script could run without it.
        let out = inject_import_map("<body><script type=\"module\"></script></body>", "{}");
        assert!(
            out.starts_with("<script type=\"importmap\">"),
            "got: {}",
            out
        );
    }

    #[test]
    fn test_content_type_by_extension() {
        assert!(content_type_for(Path::new("a.ts")).starts_with("application/javascript"));
        assert!(content_type_for(Path::new("a.tsx")).starts_with("application/javascript"));
        assert!(content_type_for(Path::new("a.js")).starts_with("application/javascript"));
        assert!(content_type_for(Path::new("a.css")).starts_with("text/css"));
        assert!(content_type_for(Path::new("a.html")).starts_with("text/html"));
        assert!(content_type_for(Path::new("a.bin")).starts_with("application/octet-stream"));
    }
}
