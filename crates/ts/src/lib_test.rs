use super::*;

#[test]
fn the_manifest_names_the_extension() {
    let manifest = TsExtension.manifest();
    assert_eq!(manifest.id, "ts");
    assert_eq!(manifest.name, "JavaScript and TypeScript");
    assert_eq!(manifest.schema_version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn nothing_is_contributed_yet() {
    let contributions = TsExtension.contributions();
    assert!(contributions.languages.is_empty());
    assert!(contributions.scaffolds.is_empty());
}
