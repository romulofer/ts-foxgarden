//! JavaScript and TypeScript support for FoxGarden, as an extension.
//!
//! Nothing is contributed yet: this is the extension's skeleton, registered
//! through `fg-extension` like every other, so the languages, grammars,
//! language servers and build tools it will bring have a place to be declared.

use fg_extension::{CURRENT_SCHEMA_VERSION, Contributions, Extension, ExtensionManifest};

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
        Contributions::default()
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod lib_test;
