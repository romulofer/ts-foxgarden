use super::*;

fn context(root: &Path, configured: &str) -> ServerStartContext {
    ServerStartContext {
        configured_binary: configured.to_string(),
        project_root: Some(root.to_path_buf()),
        ..ServerStartContext::default()
    }
}

fn install(root: &Path, relative: &str) -> PathBuf {
    let file = root.join(relative);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "").unwrap();
    file
}

#[test]
fn each_server_serves_its_languages_and_is_started_over_stdio() {
    let servers = contributions();
    let by_id = |id: &str| servers.iter().find(|s| s.id == id).unwrap();
    assert_eq!(by_id(TYPESCRIPT_SERVER).language_ids, ["javascript", "typescript", "typescriptreact"]);
    assert_eq!(by_id(HTML_SERVER).language_ids, ["html"]);
    assert_eq!(by_id(CSS_SERVER).language_ids, ["css"]);
    for server in &servers {
        assert_eq!(server.binary_name, server.id);
        assert_eq!(server.args, ["--stdio"]);
        assert!(server.install.is_none(), "the user installs {}", server.id);
    }
}

#[test]
fn without_an_install_in_the_project_the_server_is_looked_up_on_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let started = resolve(CSS_SERVER, &context(dir.path(), "")).unwrap();
    assert_eq!(started.binary, PathBuf::from("vscode-css-language-server"));
    assert_eq!(started.args, ["--stdio"]);
    assert_eq!(started.initialization_options, None);
}

#[test]
fn a_server_installed_in_the_project_is_preferred_to_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let tool = install(dir.path(), "node_modules/.bin/typescript-language-server");
    assert_eq!(resolve(TYPESCRIPT_SERVER, &context(dir.path(), "")).unwrap().binary, tool);
}

#[test]
fn a_path_configured_in_settings_wins() {
    let dir = tempfile::tempdir().unwrap();
    install(dir.path(), "node_modules/.bin/typescript-language-server");
    assert_eq!(resolve(TYPESCRIPT_SERVER, &context(dir.path(), "/opt/tsls")).unwrap().binary, PathBuf::from("/opt/tsls"));
}

#[test]
fn the_typescript_server_is_pointed_at_the_projects_own_typescript() {
    let dir = tempfile::tempdir().unwrap();
    let tsserver = install(dir.path(), "node_modules/typescript/lib/tsserver.js");
    let started = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "")).unwrap();
    let options: serde_json::Value = serde_json::from_str(&started.initialization_options.unwrap()).unwrap();
    assert_eq!(options["tsserver"]["path"], tsserver.display().to_string());
    assert_eq!(started.restart_key, tsserver.parent().unwrap().display().to_string());
}

#[test]
fn only_the_typescript_server_is_told_about_the_projects_typescript() {
    let dir = tempfile::tempdir().unwrap();
    install(dir.path(), "node_modules/typescript/lib/tsserver.js");
    assert_eq!(resolve(HTML_SERVER, &context(dir.path(), "")).unwrap().initialization_options, None);
}

#[test]
fn another_extensions_server_is_not_ours_to_start() {
    let dir = tempfile::tempdir().unwrap();
    assert!(resolve("jdtls", &context(dir.path(), "")).is_none());
}

fn project_typescript(root: &Path, version: &str, with_tsserver: bool) {
    install(root, "node_modules/typescript/package.json");
    std::fs::write(root.join("node_modules/typescript/package.json"), format!("{{\"name\":\"typescript\",\"version\":\"{version}\"}}")).unwrap();
    if with_tsserver {
        install(root, "node_modules/typescript/lib/tsserver.js");
    }
}

#[test]
fn typescript_7_is_served_by_the_projects_native_compiler() {
    let dir = tempfile::tempdir().unwrap();
    project_typescript(dir.path(), "7.0.2", false);
    let tsc = install(dir.path(), "node_modules/.bin/tsc");
    // Even with the old server installed beside it: it cannot run on 7.
    install(dir.path(), "node_modules/.bin/typescript-language-server");

    let started = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "")).unwrap();
    assert_eq!(started.binary, tsc);
    assert_eq!(started.args, ["--lsp", "--stdio"]);
    assert_eq!(started.initialization_options, None);
    assert!(started.restart_key.starts_with("native "));
}

#[test]
fn typescript_6_keeps_the_javascript_server_on_the_projects_tsserver() {
    let dir = tempfile::tempdir().unwrap();
    project_typescript(dir.path(), "6.0.3", true);
    install(dir.path(), "node_modules/.bin/tsc");
    let server = install(dir.path(), "node_modules/.bin/typescript-language-server");

    let started = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "")).unwrap();
    assert_eq!(started.binary, server);
    assert_eq!(started.args, ["--stdio"]);
    assert!(started.initialization_options.unwrap().contains("tsserver.js"));
}

#[test]
fn typescript_7_without_a_tsc_in_the_project_falls_back_to_the_javascript_server() {
    let dir = tempfile::tempdir().unwrap();
    project_typescript(dir.path(), "7.0.2", false);
    let started = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "")).unwrap();
    assert_eq!(started.binary, PathBuf::from("typescript-language-server"));
    assert_eq!(started.args, ["--stdio"]);
}

#[test]
fn a_configured_compiler_is_started_as_the_native_server() {
    let dir = tempfile::tempdir().unwrap();
    let started = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "/opt/ts/bin/tsc")).unwrap();
    assert_eq!(started.binary, PathBuf::from("/opt/ts/bin/tsc"));
    assert_eq!(started.args, ["--lsp", "--stdio"]);
    let tsgo = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "/opt/tsgo")).unwrap();
    assert_eq!(tsgo.args, ["--lsp", "--stdio"]);
}

#[test]
fn the_html_and_css_servers_ignore_the_projects_typescript() {
    let dir = tempfile::tempdir().unwrap();
    project_typescript(dir.path(), "7.0.2", false);
    install(dir.path(), "node_modules/.bin/tsc");
    let started = resolve(CSS_SERVER, &context(dir.path(), "")).unwrap();
    assert_eq!(started.binary, PathBuf::from("vscode-css-language-server"));
    assert_eq!(started.args, ["--stdio"]);
}

#[test]
fn a_project_typescript_without_a_readable_version_is_not_taken_for_7() {
    let dir = tempfile::tempdir().unwrap();
    install(dir.path(), "node_modules/typescript/package.json");
    install(dir.path(), "node_modules/.bin/tsc");
    let started = resolve(TYPESCRIPT_SERVER, &context(dir.path(), "")).unwrap();
    assert_eq!(started.binary, PathBuf::from("typescript-language-server"));
}
