use fg_extension::Registry;

use super::*;

const SAMPLES: [(&str, &str); 5] = [
    (
        JAVASCRIPT,
        "import fs from 'fs';\n\nclass Greeter {\n  hello() {\n    return <b>hi</b>;\n  }\n}\n\nfunction run() {\n  return 1;\n}\n",
    ),
    (
        TYPESCRIPT,
        "import fs from 'fs';\n\ninterface Shape {\n  area(): number;\n}\n\nenum Color {\n  Red,\n}\n\nclass Greeter {\n  hello(): string {\n    return 'hi';\n  }\n}\n\nfunction run(): number {\n  return 1;\n}\n",
    ),
    (TSX, "import React from 'react';\n\nfunction App() {\n  return <div>hi</div>;\n}\n"),
    (HTML, "<html>\n  <body>\n    <!-- note -->\n    <p>hi</p>\n  </body>\n</html>\n"),
    (CSS, "/* theme */\n.card {\n  color: red;\n}\n\n@media (min-width: 600px) {\n  .card {\n    color: blue;\n  }\n}\n"),
];

fn grammar(language_id: &str) -> GrammarContribution {
    TsExtension.contributions().grammars.into_iter().find(|g| g.language_id == language_id).unwrap()
}

fn language_of(grammar: &GrammarContribution) -> tree_sitter::Language {
    let GrammarSource::Builtin(language) = &grammar.source else { panic!("a builtin grammar") };
    (*language).into()
}

fn kinds(source: &str, language: &tree_sitter::Language) -> Vec<&'static str> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).expect("the bundled grammar must load");
    let tree = parser.parse(source, None).expect("parse");
    let mut found = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        found.push(node.kind());
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    found
}

#[test]
fn the_manifest_names_the_extension() {
    let manifest = TsExtension.manifest();
    assert_eq!(manifest.id, "ts");
    assert_eq!(manifest.name, "JavaScript and TypeScript");
    assert_eq!(manifest.schema_version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn every_language_has_a_grammar_and_an_icon() {
    let contributions = TsExtension.contributions();
    for language in &contributions.languages {
        assert!(contributions.grammars.iter().any(|g| g.language_id == language.id), "{} has no grammar", language.id);
        assert!(contributions.file_icons.iter().any(|i| i.language_id == language.id), "{} has no icon", language.id);
    }
    assert_eq!(contributions.languages.len(), 5);
}

#[test]
fn each_highlight_query_compiles_against_its_own_grammar() {
    for (id, _) in SAMPLES {
        let grammar = grammar(id);
        let query = grammar.highlight_query.clone().expect("a highlight query");
        tree_sitter::Query::new(&language_of(&grammar), &query).unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

#[test]
fn every_node_kind_the_editor_is_told_about_exists_in_a_parse_of_the_language() {
    for (id, source) in SAMPLES {
        let grammar = grammar(id);
        let found = kinds(source, &language_of(&grammar));
        // `interface_body` and `abstract_class_declaration` exist only in the
        // TypeScript grammars, and `object_type`/`enum_body` only where the
        // sample has them: the lists are shared, so only what the language's
        // sample can show is required.
        let required: &[&str] = match id {
            JAVASCRIPT => &["class_declaration", "function_declaration", "method_definition", "statement_block", "class_body", "import_statement"],
            TYPESCRIPT => &["interface_declaration", "enum_declaration", "class_declaration", "enum_body", "interface_body", "class_body", "import_statement"],
            TSX => &["function_declaration", "statement_block", "import_statement"],
            HTML => &["element", "comment"],
            _ => &["rule_set", "media_statement", "block", "comment"],
        };
        for kind in required {
            assert!(found.contains(kind), "{id}: no {kind} node in the sample's parse");
        }
        let node_kinds = grammar.node_kinds;
        for kind in required {
            assert!(
                node_kinds.scopes.iter().chain(&node_kinds.foldable).chain(node_kinds.import.iter()).any(|k| k == kind),
                "{id}: {kind} is not among the node kinds the editor is told about"
            );
        }
    }
}

#[test]
fn files_are_recognized_by_extension() {
    let mut registry = Registry::new();
    registry.register(Box::new(TsExtension)).unwrap();
    for (extension, id) in [
        ("js", JAVASCRIPT),
        ("mjs", JAVASCRIPT),
        ("cjs", JAVASCRIPT),
        ("jsx", JAVASCRIPT),
        ("ts", TYPESCRIPT),
        ("mts", TYPESCRIPT),
        ("cts", TYPESCRIPT),
        ("tsx", TSX),
        ("html", HTML),
        ("htm", HTML),
        ("css", CSS),
    ] {
        assert_eq!(registry.language_for_extension(extension).map(|l| l.language.id.as_str()), Some(id), ".{extension}");
    }
}
