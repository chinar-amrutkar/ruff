use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_python_ast as ast;
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;
use crate::importer::ImportRequest;
use crate::{Edit, Fix, FixAvailability, Violation};

/// ## What it does
/// Checks for uses of `os.path.commonprefix`.
///
/// ## Why is this bad?
/// `os.path.commonprefix` performs a character-by-character string
/// comparison rather than comparing path components. This leads to
/// incorrect results when paths share a common string prefix that
/// is not a valid path component.
///
/// `os.path.commonpath` correctly compares path components.
///
/// `os.path.commonprefix` is deprecated as of Python 3.15.
///
/// ## Semantic differences
/// `os.path.commonprefix` and `os.path.commonpath` have different semantics:
///
/// - `os.path.commonprefix` finds the longest common *string prefix* — it
///   compares characters one by one and does not consider path structure.
///   This behavior is useful for non-path string comparisons.
/// - `os.path.commonpath` finds the longest common *path component* — it
///   normalizes paths and compares them at the directory level. It requires
///   valid path-like strings and will raise `ValueError` for empty strings
///   or strings with different path types.
///
/// If you are using `os.path.commonprefix` for non-path string prefix
/// matching (e.g., finding a common prefix of arbitrary strings like
/// `"123"` and `"124"`), this rule should be ignored, as
/// `os.path.commonpath` does not support this use case.
///
/// ## Example
/// ```python
/// import os
///
/// # Returns "/usr/l" — not a valid directory!
/// os.path.commonprefix(["/usr/lib", "/usr/local/lib"])
/// ```
///
/// Use instead:
/// ```python
/// import os
///
/// # Returns "/usr" — correct common path
/// os.path.commonpath(["/usr/lib", "/usr/local/lib"])
/// ```
///
/// ## References
/// - [Python documentation: `os.path.commonprefix`](https://docs.python.org/3/library/os.path.html#os.path.commonprefix)
/// - [Python documentation: `os.path.commonpath`](https://docs.python.org/3/library/os.path.html#os.path.commonpath)
/// - [Why `os.path.commonprefix` is deprecated](https://sethmlarson.dev/deprecate-confusing-apis-like-os-path-commonprefix)
/// - [CPython deprecation issue](https://github.com/python/cpython/issues/144347)
#[derive(ViolationMetadata)]
#[violation_metadata(preview_since = "0.15.6")]
pub(crate) struct OsPathCommonprefix;

impl Violation for OsPathCommonprefix {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "`os.path.commonprefix()` compares strings character-by-character".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Use `os.path.commonpath()` to compare path components".to_string())
    }
}

/// RUF071
pub(crate) fn os_path_commonprefix(checker: &Checker, call: &ast::ExprCall, segments: &[&str]) {
    if segments != ["os", "path", "commonprefix"] {
        return;
    }
    let mut diagnostic = checker.report_diagnostic(OsPathCommonprefix, call.func.range());
    diagnostic.add_primary_tag(ruff_db::diagnostic::DiagnosticTag::Deprecated);

    diagnostic.try_set_fix(|| {
        let (import_edit, binding) = checker.importer().get_or_import_symbol(
            &ImportRequest::import_from("os.path", "commonpath"),
            call.func.start(),
            checker.semantic(),
        )?;
        let reference_edit = Edit::range_replacement(binding, call.func.range());
        Ok(Fix::unsafe_edits(import_edit, [reference_edit]))
    });
}
