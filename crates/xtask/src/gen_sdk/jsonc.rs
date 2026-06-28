//! JSONC parsing for gen_sdk — delegates to
//! [`ship_substrate::fs_util::strip_jsonc_comments`] so there is a single
//! canonical stripper for the workspace.

use anyhow::{anyhow, Result};
use serde_json::Value;

pub fn parse(src: &str) -> Result<Value> {
    let stripped = ship_substrate::fs_util::strip_jsonc_comments(src);
    serde_json::from_str(&stripped).map_err(|e| anyhow!("json parse: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_line_and_block_comments() {
        let src = r#"{
            // leading
            "a": 1, /* inline */ "b": "http://x", /* trailing */
        }"#;
        let v = parse(src).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], "http://x");
    }

    #[test]
    fn preserves_strings_with_slashes() {
        let v = parse(r#"{ "url": "a//b/*c*/" }"#).unwrap();
        assert_eq!(v["url"], "a//b/*c*/");
    }

    #[test]
    fn trailing_comma_tolerated() {
        let v = parse(r#"{ "xs": [1, 2, 3,], "y": 4, }"#).unwrap();
        assert_eq!(v["xs"][2], 3);
        assert_eq!(v["y"], 4);
    }
}
