//! JavaScript, TypeScript, HTML and CSS support for FoxGarden, as an extension.
//!
//! Contributes the five languages the web is written in — JavaScript (with
//! JSX), TypeScript, TSX, HTML and CSS — each with its tree-sitter grammar,
//! highlight query, node kinds for sticky scroll and folding, and file icon.
//! No language server, build tool or run markers yet.
//!
//! HTML and CSS belong here rather than in a framework extension: a Svelte
//! component, a Vue file and a plain page all embed them.

use fg_extension::{
    CURRENT_SCHEMA_VERSION, Contributions, Extension, ExtensionManifest, FileIcon, GrammarContribution, GrammarSource,
    LanguageContribution, LanguageId, NodeKinds,
};

pub const JAVASCRIPT: &str = "javascript";
pub const TYPESCRIPT: &str = "typescript";
pub const TSX: &str = "tsx";
pub const HTML: &str = "html";
pub const CSS: &str = "css";

pub struct TsExtension;

impl Extension for TsExtension {
    fn manifest(&self) -> ExtensionManifest {
        ExtensionManifest {
            id: "ts".to_string(),
            name: "JavaScript and TypeScript".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }

    fn contributions(&self) -> Contributions {
        Contributions {
            languages: vec![
                language(JAVASCRIPT, "JavaScript", &["js", "mjs", "cjs", "jsx"]),
                language(TYPESCRIPT, "TypeScript", &["ts", "mts", "cts"]),
                language(TSX, "TSX", &["tsx"]),
                language(HTML, "HTML", &["html", "htm"]),
                language(CSS, "CSS", &["css"]),
            ],
            grammars: vec![
                GrammarContribution {
                    language_id: JAVASCRIPT.to_string(),
                    source: GrammarSource::Builtin(tree_sitter_javascript::LANGUAGE),
                    highlight_query: Some(joined(&[
                        tree_sitter_javascript::HIGHLIGHT_QUERY,
                        tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                    ])),
                    node_kinds: script_node_kinds(),
                },
                // TypeScript's own query only covers what TypeScript adds to
                // JavaScript, so it follows JavaScript's, the way its grammar
                // extends JavaScript's.
                GrammarContribution {
                    language_id: TYPESCRIPT.to_string(),
                    source: GrammarSource::Builtin(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
                    highlight_query: Some(joined(&[
                        tree_sitter_javascript::HIGHLIGHT_QUERY,
                        tree_sitter_typescript::HIGHLIGHTS_QUERY,
                    ])),
                    node_kinds: script_node_kinds(),
                },
                GrammarContribution {
                    language_id: TSX.to_string(),
                    source: GrammarSource::Builtin(tree_sitter_typescript::LANGUAGE_TSX),
                    highlight_query: Some(joined(&[
                        tree_sitter_javascript::HIGHLIGHT_QUERY,
                        tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                        tree_sitter_typescript::HIGHLIGHTS_QUERY,
                    ])),
                    node_kinds: script_node_kinds(),
                },
                GrammarContribution {
                    language_id: HTML.to_string(),
                    source: GrammarSource::Builtin(tree_sitter_html::LANGUAGE),
                    highlight_query: Some(tree_sitter_html::HIGHLIGHTS_QUERY.to_string()),
                    node_kinds: html_node_kinds(),
                },
                GrammarContribution {
                    language_id: CSS.to_string(),
                    source: GrammarSource::Builtin(tree_sitter_css::LANGUAGE),
                    highlight_query: Some(tree_sitter_css::HIGHLIGHTS_QUERY.to_string()),
                    node_kinds: css_node_kinds(),
                },
            ],
            // Nerd Fonts devicons and seti glyphs.
            file_icons: vec![
                icon(JAVASCRIPT, '\u{e74e}'),
                icon(TYPESCRIPT, '\u{e628}'),
                icon(TSX, '\u{e7ba}'),
                icon(HTML, '\u{e736}'),
                icon(CSS, '\u{e749}'),
            ],
            ..Contributions::default()
        }
    }
}

fn language(id: &str, display_name: &str, extensions: &[&str]) -> LanguageContribution {
    LanguageContribution {
        id: LanguageId::from(id),
        display_name: display_name.to_string(),
        file_extensions: extensions.iter().map(|e| (*e).to_string()).collect(),
        filename_patterns: Vec::new(),
    }
}

fn icon(language_id: &str, glyph: char) -> FileIcon {
    FileIcon { language_id: language_id.to_string(), glyph }
}

fn joined(queries: &[&str]) -> String {
    queries.join("\n")
}

fn names(kinds: &[&str]) -> Vec<String> {
    kinds.iter().map(|k| (*k).to_string()).collect()
}

/// Shared by JavaScript, TypeScript and TSX, whose grammars name these alike:
/// a declaration's header is pinned while its body scrolls, and the bodies fold.
/// `interface_body` is TypeScript's; JavaScript's grammar has no such node,
/// which is harmless to list.
fn script_node_kinds() -> NodeKinds {
    NodeKinds {
        scopes: names(&[
            "class_declaration",
            "abstract_class_declaration",
            "interface_declaration",
            "enum_declaration",
            "function_declaration",
            "generator_function_declaration",
            "method_definition",
        ]),
        foldable: names(&["statement_block", "class_body", "interface_body", "object_type", "enum_body", "switch_body", "comment"]),
        import: Some("import_statement".to_string()),
    }
}

/// An element folds from its start tag to its end tag; no element is pinned,
/// since a page's nesting is deep and its opening tags say little.
fn html_node_kinds() -> NodeKinds {
    NodeKinds { scopes: Vec::new(), foldable: names(&["element", "comment"]), import: None }
}

/// A rule pins its selector line; a block, an at-rule's body or a comment folds.
fn css_node_kinds() -> NodeKinds {
    NodeKinds { scopes: names(&["rule_set", "media_statement"]), foldable: names(&["block", "comment"]), import: None }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod lib_test;
