//! Clipboard operations (Phase 19).
//!
//! Provides copy, paste, and read operations for the browser clipboard.

use serde::{Deserialize, Serialize};

/// Clipboard action type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardAction {
    /// Copy selected content to clipboard.
    Copy,
    /// Paste clipboard content.
    Paste,
    /// Read clipboard content (without pasting).
    Read,
    /// Write text to clipboard.
    Write,
}

/// Result of a clipboard operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardResult {
    /// The clipboard text content (for read operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Execute a clipboard operation via JavaScript in the page context.
///
/// Returns the JavaScript expression for the given clipboard action.
pub fn clipboard_js(action: ClipboardAction, text: Option<&str>) -> String {
    match action {
        ClipboardAction::Copy => r#"
            (async function() {
                try {
                    const selection = window.getSelection();
                    if (selection && selection.toString()) {
                        await navigator.clipboard.writeText(selection.toString());
                        return { success: true, text: selection.toString() };
                    }
                    // Fallback: trigger copy command
                    document.execCommand('copy');
                    return { success: true, text: null };
                } catch (e) {
                    return { success: false, text: null };
                }
            })()
            "#
        .to_string(),
        ClipboardAction::Paste => r#"
            (async function() {
                try {
                    const text = await navigator.clipboard.readText();
                    document.execCommand('insertText', false, text);
                    return { success: true, text: text };
                } catch (e) {
                    // Fallback: trigger paste command
                    document.execCommand('paste');
                    return { success: true, text: null };
                }
            })()
            "#
        .to_string(),
        ClipboardAction::Read => r#"
            (async function() {
                try {
                    const text = await navigator.clipboard.readText();
                    return { success: true, text: text };
                } catch (e) {
                    return { success: false, text: null };
                }
            })()
            "#
        .to_string(),
        ClipboardAction::Write => {
            let escaped = text
                .unwrap_or("")
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
                .replace('\n', "\\n");
            format!(
                r#"
                (async function() {{
                    try {{
                        await navigator.clipboard.writeText('{}');
                        return {{ success: true, text: '{}' }};
                    }} catch (e) {{
                        return {{ success: false, text: null }};
                    }}
                }})()
                "#,
                escaped, escaped
            )
        }
    }
}

/// Grant clipboard permissions via CDP.
pub fn grant_clipboard_permissions_js() -> &'static str {
    r#"
    (function() {
        // Ensure clipboard API is available by granting permissions
        if (navigator.permissions && navigator.permissions.query) {
            navigator.permissions.query({ name: 'clipboard-read' }).catch(() => {});
            navigator.permissions.query({ name: 'clipboard-write' }).catch(() => {});
        }
    })()
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_action_serialize() {
        let copy = serde_json::to_string(&ClipboardAction::Copy).unwrap();
        assert_eq!(copy, "\"copy\"");

        let paste = serde_json::to_string(&ClipboardAction::Paste).unwrap();
        assert_eq!(paste, "\"paste\"");

        let read = serde_json::to_string(&ClipboardAction::Read).unwrap();
        assert_eq!(read, "\"read\"");

        let write = serde_json::to_string(&ClipboardAction::Write).unwrap();
        assert_eq!(write, "\"write\"");
    }

    #[test]
    fn test_clipboard_js_generation() {
        let copy_js = clipboard_js(ClipboardAction::Copy, None);
        assert!(copy_js.contains("clipboard.writeText"));

        let paste_js = clipboard_js(ClipboardAction::Paste, None);
        assert!(paste_js.contains("clipboard.readText"));

        let read_js = clipboard_js(ClipboardAction::Read, None);
        assert!(read_js.contains("clipboard.readText"));

        let write_js = clipboard_js(ClipboardAction::Write, Some("hello world"));
        assert!(write_js.contains("hello world"));
    }

    #[test]
    fn test_clipboard_result_serialize() {
        let result = ClipboardResult {
            text: Some("hello".to_string()),
            success: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"text\":\"hello\""));
        assert!(json.contains("\"success\":true"));
    }
}
