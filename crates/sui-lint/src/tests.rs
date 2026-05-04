//! Tests for SUI lint rules.

use std::path::Path;

use crate::{is_controller_path, lint_controller_source, rules::controller_purity};

fn lint(source: &str) -> Vec<crate::Finding> {
    lint_controller_source(Path::new("/fake/foo.controller.ts"), source)
}

#[test]
fn clean_controller_passes() {
    let src = r#"
        import { controller, signal } from '@ship/ui-runtime'
        export default controller({
          mount: (state, host) => {
            const count = signal(0)
            host.addEventListener('click', () => count.set(count() + 1))
            return () => {}
          },
        })
    "#;
    let findings = lint(src);
    assert!(findings.is_empty(), "clean controller should pass: {findings:?}");
}

#[test]
fn inner_html_assignment_reports() {
    let src = r#"
        export default {
          mount: (_state, host) => {
            host.innerHTML = '<p>hi</p>'
            return () => {}
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1, "expected one finding, got {findings:?}");
    assert_eq!(findings[0].rule, controller_purity::RULE_ID);
    assert!(findings[0].message.contains(".innerHTML"), "msg: {}", findings[0].message);
    assert_eq!(findings[0].line, 4);
    assert!(findings[0].suggest.is_some(), "should suggest bindHtml");
}

#[test]
fn outer_html_assignment_reports() {
    let src = r#"
        export default {
          mount: (_state, host) => {
            host.outerHTML = '<p>hi</p>'
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains(".outerHTML"));
}

#[test]
fn insert_adjacent_html_reports() {
    let src = r#"
        export default {
          mount: (_s, host) => {
            host.insertAdjacentHTML('beforeend', '<p>hi</p>')
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1, "got {findings:?}");
    assert!(findings[0].message.contains("insertAdjacentHTML"));
}

#[test]
fn nested_member_chain_reports() {
    // host.shadowRoot.innerHTML = … is the common shadow-DOM slop.
    let src = r#"
        export default {
          mount: (_s, host) => {
            host.shadowRoot.innerHTML = '<div></div>'
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1);
}

#[test]
fn text_content_passes() {
    // Setting textContent is fine — it's not HTML, and it's how
    // controllers should update text from outside the template.
    let src = r#"
        export default {
          mount: (_s, host) => {
            host.textContent = 'hello'
          },
        }
    "#;
    let findings = lint(src);
    assert!(findings.is_empty(), "textContent must be allowed: {findings:?}");
}

#[test]
fn computed_member_with_innerhtml_string_passes() {
    // host['inner' + 'HTML'] = … is computed access — we can't
    // statically know it's innerHTML. We don't report; if someone is
    // doing this they're already pathologically working around the
    // rule and tsc/runtime will catch it elsewhere.
    let src = r#"
        export default {
          mount: (_s, host) => {
            host['innerHTML'] = '<p>hi</p>'
          },
        }
    "#;
    // Computed member with a literal — rule doesn't fire (we only
    // match StaticMemberExpression). Acceptable for a hard ban now;
    // can tighten later if needed.
    let findings = lint(src);
    assert!(findings.is_empty(), "computed access not currently inspected");
}

#[test]
fn parse_error_returns_empty() {
    // Invalid TS: parse error → rule swallows, returns no findings.
    let findings = lint("export default { mount: (host =>");
    assert!(findings.is_empty());
}

#[test]
fn is_controller_path_recognises_suffix() {
    assert!(is_controller_path(Path::new("foo/bar.controller.ts")));
    assert!(is_controller_path(Path::new("/abs/sui-button.controller.ts")));
    assert!(!is_controller_path(Path::new("foo/bar.ts")));
    assert!(!is_controller_path(Path::new("foo/bar.controller.tsx")));
    assert!(!is_controller_path(Path::new("foo/bar.sui")));
    assert!(!is_controller_path(Path::new("foo/render.ts")));
}

#[test]
fn flags_real_offender_sui_command_bar() {
    // Smoke against the actual file in-repo. The QA established that
    // sui-command-bar.controller.ts has at least one .innerHTML write.
    // If the file moves or the rule changes, this test re-points or
    // gets removed — it's a sanity check, not a contract.
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("packages/ui-components/src/sui-command-bar/sui-command-bar.controller.ts");
    if !path.exists() {
        eprintln!("skipping: {} not present", path.display());
        return;
    }
    let source = std::fs::read_to_string(&path).expect("read sui-command-bar controller");
    let findings = lint_controller_source(&path, &source);
    assert!(
        !findings.is_empty(),
        "expected sui-command-bar.controller.ts to report at least one violation; \
         got none. Either the file's been cleaned up (good!) or the rule regressed."
    );
}
