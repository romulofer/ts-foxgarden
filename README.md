# ts-foxgarden

JavaScript and TypeScript support for [FoxGarden](../foxgarden), as an extension
rather than as part of the editor.

It reaches the editor only through `fg-extension`, like
[`spring-foxgarden`](../spring-foxgarden) and [`kmp-foxgarden`](../kmp-foxgarden).
This checkout must sit beside `foxgarden`.

## Status

Skeleton only: the crate registers an extension with an id and a name, and
contributes nothing yet. It is not wired into `foxgarden`'s `crates/languages`.

## Development

```sh
cargo test --workspace
```
