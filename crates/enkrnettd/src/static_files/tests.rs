//! ENK-API-024, ENK-API-026, ENK-MGMT-005.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct UiDir(PathBuf);

impl UiDir {
    fn new() -> UiDir {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("enkrnettd-ui-test-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("assets")).unwrap();
        std::fs::write(p.join("index.html"), "<!doctype html><title>UI</title>").unwrap();
        std::fs::write(p.join("assets/app-0123abcd.js"), "console.log(1)").unwrap();
        std::fs::write(p.join("favicon.svg"), "<svg/>").unwrap();
        std::fs::write(p.join("secret.txt"), "outside").unwrap();
        UiDir(p)
    }
}

impl Drop for UiDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn header<'a>(r: &'a Response, name: &str) -> Option<&'a str> {
    r.headers
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

#[test]
fn entry_page_is_no_store_and_hashed_files_are_immutable() {
    let dir = UiDir::new();
    let s = StaticFiles::new(Some(dir.0.clone()));
    let r = s.get("/");
    assert_eq!(r.status, 200);
    assert_eq!(header(&r, "Cache-Control"), Some("no-store"));
    assert_eq!(header(&r, "Content-Type"), Some("text/html; charset=utf-8"));
    let r = s.get("/index.html");
    assert_eq!(header(&r, "Cache-Control"), Some("no-store"));
    let r = s.get("/assets/app-0123abcd.js");
    assert_eq!(r.status, 200);
    assert_eq!(
        header(&r, "Cache-Control"),
        Some("max-age=31536000, immutable")
    );
    assert_eq!(
        header(&r, "Content-Type"),
        Some("text/javascript; charset=utf-8")
    );
    let r = s.get("/favicon.svg");
    assert_eq!(header(&r, "Cache-Control"), Some("no-cache"));
}

#[test]
fn client_routes_fall_back_to_the_entry_page_but_missing_assets_do_not() {
    let dir = UiDir::new();
    let s = StaticFiles::new(Some(dir.0.clone()));
    let r = s.get("/settings");
    assert_eq!(r.status, 200);
    assert_eq!(header(&r, "Cache-Control"), Some("no-store"));
    assert!(String::from_utf8_lossy(&r.body).contains("<title>UI</title>"));
    let r = s.get("/assets/missing-0123abcd.js");
    assert_eq!(r.status, 404);
}

#[test]
fn traversal_is_refused() {
    let dir = UiDir::new();
    let s = StaticFiles::new(Some(dir.0.clone()));
    assert_eq!(s.get("/../secret.txt").status, 404);
    assert_eq!(s.get("/%2e%2e/secret.txt").status, 404);
    assert_eq!(s.get("/assets/../../secret.txt").status, 404);
    assert_eq!(sanitize("/a/./b"), Some(PathBuf::from("a/b")));
    assert_eq!(sanitize("/a/%00"), None);
    assert_eq!(sanitize("/%zz"), None);
}

#[test]
fn hash_detection() {
    assert!(is_content_hashed("app-0123abcd.js"));
    assert!(is_content_hashed("app.0123ABCDef.css"));
    assert!(!is_content_hashed("app-0123abc.js"));
    assert!(!is_content_hashed("index.html"));
    assert!(!is_content_hashed("nohash"));
}

#[test]
fn without_a_ui_directory_a_placeholder_is_served() {
    let s = StaticFiles::new(None);
    let r = s.get("/anything");
    assert_eq!(r.status, 200);
    assert!(String::from_utf8_lossy(&r.body).contains("not installed"));
}
