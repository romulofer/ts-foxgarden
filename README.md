# ts-foxgarden

JavaScript, TypeScript, HTML and CSS support for [FoxGarden](../foxgarden), as an
extension rather than as part of the editor.

It reaches the editor only through `fg-extension`, like
[`spring-foxgarden`](../spring-foxgarden) and [`kmp-foxgarden`](../kmp-foxgarden).
This checkout must sit beside `foxgarden`, which compiles it in.

HTML and CSS live here rather than in a framework extension: a Svelte
component, a Vue file and a plain page all embed them.

## What it contributes today

| Language | Extensions | Language server |
|---|---|---|
| `javascript` | `.js` `.mjs` `.cjs` `.jsx` | `typescript-language-server` |
| `typescript` | `.ts` `.mts` `.cts` | `typescript-language-server` |
| `typescriptreact` (TSX) | `.tsx` | `typescript-language-server` |
| `html` | `.html` `.htm` | `vscode-html-language-server` |
| `css` | `.css` | `vscode-css-language-server` |

Each has its tree-sitter grammar with the grammar crate's own highlight query
(TypeScript's follow JavaScript's, as its grammar extends it), node kinds for
sticky scroll and folding, and a file icon. TSX's id is the name the Language
Server Protocol uses, because the id is also the `languageId` a server is told a
document has.

## Language servers

None is bundled; they are npm packages. FoxGarden looks for each in the project's
`node_modules/.bin`, then on `PATH`; a path set in Settings ▸ Language Servers…
wins over both.

```sh
npm install --save-dev typescript typescript-language-server   # JavaScript and TypeScript up to 6
npm install --save-dev typescript                              # TypeScript 7
npm install --save-dev vscode-langservers-extracted            # HTML and CSS
```

The JavaScript/TypeScript server follows the TypeScript the project is built
with (`node_modules/typescript/package.json`):

- **TypeScript 7 or newer** is a native compiler with no `tsserver.js`, so
  `typescript-language-server` cannot run on it. The project's own
  `node_modules/.bin/tsc --lsp --stdio` is the server instead.
- **TypeScript 6 and older** run `typescript-language-server`, pointed at the
  project's `tsserver.js` so the types match the version the project uses.
- With no TypeScript in the project, `typescript-language-server` from `PATH`.
- A configured path to `tsc` or `tsgo` is started as the native server.

## Not here yet

An installer for the servers, ESLint and Prettier, the JSON and Markdown servers
from `vscode-langservers-extracted`, CSS and JavaScript embedded in HTML (the
editor has no language injections), a build tool and run markers (`npm`
scripts, `node`).

## Development

```sh
cargo test --workspace
```

The servers are exercised end to end by `#[ignore]`d tests in `foxgarden`
(`FOXGARDEN_NODE_MODULES=<a node_modules with the servers> cargo test -p foxgarden
--bin foxgarden -- --ignored typescript_completion css_completion html_completion`).
