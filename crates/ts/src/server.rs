//! The language servers: `typescript-language-server` for JavaScript and
//! TypeScript, and the VS Code HTML and CSS servers
//! (`vscode-langservers-extracted`) for HTML and CSS.
//!
//! None is bundled; they are npm packages the user installs, in the project
//! (`npm install --save-dev typescript typescript-language-server`) or
//! globally. A project on TypeScript 7 needs only `typescript`: its native
//! compiler is the server. Each is found in the project's `node_modules/.bin` first, then on
//! `PATH`, and a path configured in Settings wins over both.

use std::path::{Path, PathBuf};

use fg_extension::{LanguageServerContribution, ResolvedServerStart, ServerStartContext};

use crate::{CSS, HTML, JAVASCRIPT, TSX, TYPESCRIPT};

pub const TYPESCRIPT_SERVER: &str = "typescript-language-server";
pub const HTML_SERVER: &str = "vscode-html-language-server";
pub const CSS_SERVER: &str = "vscode-css-language-server";

fn server(id: &str, display_name: &str, language_ids: &[&str]) -> LanguageServerContribution {
    LanguageServerContribution {
        id: id.to_string(),
        display_name: display_name.to_string(),
        language_ids: language_ids.iter().map(|id| (*id).to_string()).collect(),
        // Each server's program is named after it.
        binary_name: id.to_string(),
        args: vec!["--stdio".to_string()],
        initialization_options: None,
        install: None,
        virtual_documents: None,
        host_runtime: None,
    }
}

pub(crate) fn contributions() -> Vec<LanguageServerContribution> {
    vec![
        server(TYPESCRIPT_SERVER, "TypeScript Language Server", &[JAVASCRIPT, TYPESCRIPT, TSX]),
        server(HTML_SERVER, "HTML Language Server", &[HTML]),
        server(CSS_SERVER, "CSS Language Server", &[CSS]),
    ]
}

/// A program installed into the project (`node_modules/.bin/<name>`, with the
/// `.cmd` shim npm writes on Windows).
fn project_program(project_root: &Path, name: &str) -> Option<PathBuf> {
    let bin = project_root.join("node_modules").join(".bin");
    [name.to_string(), format!("{name}.cmd")].into_iter().map(|file| bin.join(file)).find(|candidate| candidate.is_file())
}

/// The major version of the TypeScript the project has installed, from its
/// `node_modules/typescript/package.json`.
fn project_typescript_major(project_root: &Path) -> Option<u32> {
    let manifest = std::fs::read_to_string(project_root.join("node_modules").join("typescript").join("package.json")).ok()?;
    let version = serde_json::from_str::<serde_json::Value>(&manifest).ok()?["version"].as_str()?.to_string();
    version.split('.').next()?.parse().ok()
}

/// Whether `binary` is TypeScript's own compiler, which since TypeScript 7 is a
/// native program that is also its language server (`tsc --lsp --stdio`).
fn is_native_compiler(binary: &Path) -> bool {
    matches!(binary.file_stem().and_then(|stem| stem.to_str()), Some("tsc" | "tsgo"))
}

/// How to start `server_id` for the project in `context`, or `None` for a
/// server this extension does not own.
///
/// The TypeScript server follows the TypeScript the project is built with. From
/// version 7 the compiler is native and ships no `tsserver.js`, so
/// `typescript-language-server` cannot run on it; the project's own `tsc` is the
/// server then. Up to version 6 (and with no TypeScript in the project) it is
/// `typescript-language-server`, pointed at the project's `tsserver.js` when it
/// has one, so types and language features match the version the project uses.
pub(crate) fn resolve(server_id: &str, context: &ServerStartContext) -> Option<ResolvedServerStart> {
    let owned = contributions().into_iter().find(|server| server.id == server_id)?;
    let root = context.project_root.as_deref();
    let configured = context.configured_binary.trim();
    let is_typescript = server_id == TYPESCRIPT_SERVER;

    let native = (is_typescript && configured.is_empty())
        .then(|| root.filter(|root| project_typescript_major(root).is_some_and(|major| major >= 7)))
        .flatten()
        .and_then(|root| project_program(root, "tsc"));
    let binary = if !configured.is_empty() {
        PathBuf::from(configured)
    } else if let Some(tsc) = native {
        tsc
    } else {
        root.and_then(|root| project_program(root, &owned.binary_name)).unwrap_or_else(|| PathBuf::from(&owned.binary_name))
    };

    if is_native_compiler(&binary) {
        return Some(ResolvedServerStart {
            args: vec!["--lsp".to_string(), "--stdio".to_string()],
            restart_key: format!("native {}", binary.display()),
            binary,
            initialization_options: None,
        });
    }
    let tsserver = is_typescript
        .then(|| root.map(|root| root.join("node_modules").join("typescript").join("lib")).filter(|lib| lib.join("tsserver.js").is_file()))
        .flatten();
    Some(ResolvedServerStart {
        binary,
        args: owned.args,
        initialization_options: tsserver.as_deref().map(initialization_options),
        // A session started before the project installed its TypeScript is
        // replaced by one that uses it.
        restart_key: tsserver.map(|lib| lib.display().to_string()).unwrap_or_default(),
    })
}

fn initialization_options(lib: &Path) -> String {
    serde_json::json!({ "tsserver": { "path": lib.join("tsserver.js").display().to_string() } }).to_string()
}

#[cfg(test)]
#[path = "server_test.rs"]
mod server_test;
