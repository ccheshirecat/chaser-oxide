//! Snapshot system for generating accessibility trees with refs.
//!
//! The snapshot functionality generates a streamlined accessibility tree where
//! each interactive element gets a unique ref (`@e1`, `@e2`, etc.). This dramatically
//! reduces the amount of data an AI agent needs to process while maintaining
//! full interactivity.

use super::refs::{ElementBounds, RefId, RefInfo, RefMap};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Options for snapshot generation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SnapshotOptions {
    /// Only include interactive elements (buttons, links, inputs, etc.).
    pub interactive_only: bool,

    /// Compact mode: remove unnamed structural elements.
    pub compact: bool,

    /// Maximum depth to traverse (0 = unlimited).
    pub max_depth: usize,

    /// CSS selector to scope the snapshot to.
    pub scope_selector: Option<String>,

    /// Include hidden elements.
    pub include_hidden: bool,

    /// Include element values (input contents, etc.).
    pub include_values: bool,
}

impl SnapshotOptions {
    /// Create default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Only include interactive elements (-i flag).
    pub fn interactive_only(mut self) -> Self {
        self.interactive_only = true;
        self
    }

    /// Enable compact mode (-c flag).
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Set maximum depth (-d flag).
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Scope to a CSS selector (-s flag).
    pub fn scope(mut self, selector: impl Into<String>) -> Self {
        self.scope_selector = Some(selector.into());
        self
    }

    /// Include hidden elements.
    pub fn include_hidden(mut self) -> Self {
        self.include_hidden = true;
        self
    }

    /// Include element values.
    pub fn include_values(mut self) -> Self {
        self.include_values = true;
        self
    }
}

/// An accessibility tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNode {
    /// The accessibility role (button, textbox, heading, etc.).
    pub role: String,

    /// The accessible name (label, text content, aria-label).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The assigned ref ID for interactive elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<RefId>,

    /// Child nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<AccessibilityNode>,

    /// Backend node ID from CDP.
    #[serde(skip)]
    pub backend_node_id: Option<i64>,

    /// Whether this node is interactive.
    #[serde(skip)]
    pub interactive: bool,

    /// Whether this node is visible.
    #[serde(skip)]
    pub visible: bool,

    /// Value for input elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Checked state for checkboxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,

    /// Disabled state.
    #[serde(skip)]
    pub disabled: bool,

    /// Level for headings (1-6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,

    /// Bounding box.
    #[serde(skip)]
    pub bounds: Option<ElementBounds>,

    /// Depth in the tree.
    #[serde(skip)]
    pub depth: usize,
}

impl AccessibilityNode {
    /// Create a new node with just a role.
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            name: None,
            ref_id: None,
            children: Vec::new(),
            backend_node_id: None,
            interactive: false,
            visible: true,
            value: None,
            checked: None,
            disabled: false,
            level: None,
            bounds: None,
            depth: 0,
        }
    }

    /// Set the name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if !name.is_empty() {
            self.name = Some(name);
        }
        self
    }

    /// Set the backend node ID.
    pub fn with_backend_node_id(mut self, id: i64) -> Self {
        self.backend_node_id = Some(id);
        self
    }

    /// Mark as interactive.
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    /// Set visibility.
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: AccessibilityNode) {
        self.children.push(child);
    }

    /// Check if this node should get a ref.
    pub fn should_have_ref(&self) -> bool {
        self.interactive && self.visible && !self.disabled
    }

    /// Format as a tree string with indentation.
    pub fn to_tree_string(&self, options: &SnapshotOptions) -> String {
        let mut output = String::new();
        self.format_tree_recursive(&mut output, 0, options);
        output
    }

    fn format_tree_recursive(&self, output: &mut String, indent: usize, options: &SnapshotOptions) {
        // Skip if filtered out
        if options.interactive_only && !self.interactive && self.children.is_empty() {
            return;
        }

        if options.compact && !self.interactive && self.name.is_none() && self.children.len() <= 1 {
            // In compact mode, skip unnamed structural nodes with 0-1 children
            for child in &self.children {
                child.format_tree_recursive(output, indent, options);
            }
            return;
        }

        // Check depth limit
        if options.max_depth > 0 && indent >= options.max_depth {
            return;
        }

        // Build the line
        let indent_str = "  ".repeat(indent);

        // Format: [role] name @ref
        let mut line = format!("{}[{}]", indent_str, self.role);

        // Add level for headings
        if let Some(level) = self.level {
            line.push_str(&format!(" level={}", level));
        }

        // Add name
        if let Some(ref name) = self.name {
            line.push(' ');
            line.push_str(name);
        }

        // Add ref
        if let Some(ref ref_id) = self.ref_id {
            line.push(' ');
            line.push_str(&ref_id.display());
        }

        // Add value for inputs
        if options.include_values {
            if let Some(ref value) = self.value {
                if !value.is_empty() {
                    line.push_str(&format!(" value=\"{}\"", truncate_value(value, 50)));
                }
            }
        }

        // Add checked state
        if let Some(checked) = self.checked {
            line.push_str(if checked { " [x]" } else { " [ ]" });
        }

        output.push_str(&line);
        output.push('\n');

        // Recurse to children
        for child in &self.children {
            child.format_tree_recursive(output, indent + 1, options);
        }
    }

    /// Count total nodes in this subtree.
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }

    /// Count interactive nodes in this subtree.
    pub fn count_interactive(&self) -> usize {
        let self_count = if self.interactive { 1 } else { 0 };
        self_count
            + self
                .children
                .iter()
                .map(|c| c.count_interactive())
                .sum::<usize>()
    }
}

/// A complete snapshot of the page's accessibility tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// The formatted tree string (for AI consumption).
    pub tree: String,

    /// The ref map for looking up element info.
    #[serde(skip)]
    pub refs: RefMap,

    /// The root node (for programmatic access).
    #[serde(skip)]
    pub root: Option<AccessibilityNode>,

    /// Total number of nodes in the tree.
    pub total_nodes: usize,

    /// Number of interactive elements (with refs).
    pub interactive_count: usize,

    /// Page URL when snapshot was taken.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Page title when snapshot was taken.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Snapshot {
    /// Create a new snapshot from a root node.
    pub fn from_tree(mut root: AccessibilityNode, options: &SnapshotOptions) -> Self {
        let mut refs = RefMap::new();

        // Assign refs to interactive elements
        assign_refs_recursive(&mut root, &mut refs);

        let total_nodes = root.count_nodes();
        let interactive_count = refs.len();
        let tree = root.to_tree_string(options);

        Self {
            tree,
            refs,
            root: Some(root),
            total_nodes,
            interactive_count,
            url: None,
            title: None,
        }
    }

    /// Create an empty snapshot.
    pub fn empty() -> Self {
        Self {
            tree: String::new(),
            refs: RefMap::new(),
            root: None,
            total_nodes: 0,
            interactive_count: 0,
            url: None,
            title: None,
        }
    }

    /// Set page URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set page title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Get the ref map for element lookup.
    pub fn ref_map(&self) -> &RefMap {
        &self.refs
    }

    /// Look up a ref by its string representation.
    pub fn lookup(&self, ref_str: &str) -> Option<&RefInfo> {
        self.refs.lookup(ref_str)
    }

    /// Get all interactive refs.
    pub fn interactive_refs(&self) -> Vec<(&RefId, &RefInfo)> {
        self.refs.interactive()
    }
}

/// Recursively assign refs to interactive nodes.
fn assign_refs_recursive(node: &mut AccessibilityNode, refs: &mut RefMap) {
    if node.should_have_ref() {
        let info = RefInfo {
            role: node.role.clone(),
            name: node.name.clone(),
            selector: None,
            nth: 0,
            backend_node_id: node.backend_node_id,
            xpath: None,
            interactive: true,
            visible: node.visible,
            bounds: node.bounds,
            value: node.value.clone(),
            checked: node.checked,
            disabled: node.disabled,
            level: node.level,
            attributes: Default::default(),
        };
        let ref_id = refs.add(info);
        node.ref_id = Some(ref_id);
    }

    for child in &mut node.children {
        assign_refs_recursive(child, refs);
    }
}

/// Roles that are considered interactive.
pub static INTERACTIVE_ROLES: &[&str] = &[
    "button",
    "link",
    "textbox",
    "searchbox",
    "checkbox",
    "radio",
    "combobox",
    "listbox",
    "option",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "tab",
    "slider",
    "spinbutton",
    "switch",
    "treeitem",
    "gridcell",
    "columnheader",
    "rowheader",
];

/// Check if a role is considered interactive.
pub fn is_interactive_role(role: &str) -> bool {
    let role_lower = role.to_lowercase();
    INTERACTIVE_ROLES.iter().any(|r| *r == role_lower)
}

/// Roles that should be included even in compact mode.
/// TODO: Use this for compact mode filtering in a future phase.
#[allow(dead_code)]
pub static IMPORTANT_ROLES: &[&str] = &[
    "heading",
    "navigation",
    "main",
    "banner",
    "contentinfo",
    "complementary",
    "form",
    "search",
    "region",
    "alert",
    "alertdialog",
    "dialog",
    "status",
    "img",
    "figure",
    "table",
    "list",
    "listitem",
    "article",
];

/// Truncate a value for display.
fn truncate_value(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.replace('\n', "\\n").replace('\r', "")
    } else {
        format!(
            "{}...",
            &s[..max_len].replace('\n', "\\n").replace('\r', "")
        )
    }
}

/// Filter for which nodes to include based on options.
/// TODO: Use this for more sophisticated filtering in a future phase.
#[allow(dead_code)]
pub struct SnapshotFilter {
    interactive_only: bool,
    compact: bool,
    max_depth: usize,
    seen_roles: HashSet<String>,
}

#[allow(dead_code)]
impl SnapshotFilter {
    /// Create a new filter from options.
    pub fn new(options: &SnapshotOptions) -> Self {
        Self {
            interactive_only: options.interactive_only,
            compact: options.compact,
            max_depth: options.max_depth,
            seen_roles: HashSet::new(),
        }
    }

    /// Check if a node should be included.
    pub fn should_include(&self, node: &AccessibilityNode) -> bool {
        // Always include interactive elements
        if node.interactive {
            return true;
        }

        // Check depth
        if self.max_depth > 0 && node.depth >= self.max_depth {
            return false;
        }

        // Interactive-only mode
        if self.interactive_only {
            // Include if has interactive descendants
            return node.children.iter().any(|c| self.should_include(c));
        }

        // Compact mode
        if self.compact {
            // Include if named or important role
            let role_lower = node.role.to_lowercase();
            if node.name.is_some() || IMPORTANT_ROLES.iter().any(|r| *r == role_lower) {
                return true;
            }
            // Include if has multiple children
            if node.children.len() > 1 {
                return true;
            }
            // Include if has interactive descendants
            return node.children.iter().any(|c| self.should_include(c));
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_tree_string() {
        let mut root = AccessibilityNode::new("document").with_name("Test Page");

        let mut form = AccessibilityNode::new("form").with_name("Login");

        let mut email = AccessibilityNode::new("textbox")
            .with_name("Email")
            .interactive();
        email.ref_id = Some(RefId::new(1));

        let mut password = AccessibilityNode::new("textbox")
            .with_name("Password")
            .interactive();
        password.ref_id = Some(RefId::new(2));

        let mut submit = AccessibilityNode::new("button")
            .with_name("Sign In")
            .interactive();
        submit.ref_id = Some(RefId::new(3));

        form.add_child(email);
        form.add_child(password);
        form.add_child(submit);
        root.add_child(form);

        let options = SnapshotOptions::default();
        let tree = root.to_tree_string(&options);

        assert!(tree.contains("[document] Test Page"));
        assert!(tree.contains("[form] Login"));
        assert!(tree.contains("[textbox] Email @e1"));
        assert!(tree.contains("[textbox] Password @e2"));
        assert!(tree.contains("[button] Sign In @e3"));
    }

    #[test]
    fn test_interactive_roles() {
        assert!(is_interactive_role("button"));
        assert!(is_interactive_role("BUTTON"));
        assert!(is_interactive_role("textbox"));
        assert!(!is_interactive_role("document"));
        assert!(!is_interactive_role("generic"));
    }

    #[test]
    fn test_snapshot_from_tree() {
        let mut root = AccessibilityNode::new("document");
        let btn = AccessibilityNode::new("button")
            .with_name("Click me")
            .interactive();
        root.add_child(btn);

        let snapshot = Snapshot::from_tree(root, &SnapshotOptions::default());

        assert_eq!(snapshot.interactive_count, 1);
        assert!(snapshot.lookup("@e1").is_some());
        assert!(snapshot.tree.contains("[button] Click me @e1"));
    }
}
