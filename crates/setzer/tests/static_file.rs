use setzer::StaticFile;

const MANIFEST: StaticFile = setzer::static_file!("Cargo.toml");
const MOVED: StaticFile = setzer::static_file!("Cargo.toml" => "assets/text/manifest.toml");

#[test]
fn served_name_carries_the_content_hash() {
    let hash = MANIFEST
        .path
        .strip_prefix("Cargo-")
        .and_then(|n| n.strip_suffix(".toml"))
        .unwrap();
    assert_eq!(hash.len(), 8);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(MANIFEST.bytes, include_bytes!("../Cargo.toml"));
    assert_eq!(MANIFEST.url_path(), format!("/{}", MANIFEST.path));
    assert_eq!(MOVED.path, format!("assets/text/manifest-{hash}.toml"));
}
