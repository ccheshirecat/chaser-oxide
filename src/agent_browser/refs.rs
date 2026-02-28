//! Reference system for the Snapshot + Refs innovation.
//!
//! This module implements the core innovation of agent-browser: element references
//! that dramatically reduce AI context usage by providing stable identifiers
//! (`@e1`, `@e2`, etc.) instead of full DOM paths or complex selectors.
//!
//! # How Refs Work
//!
//! 1. When you call `snapshot()`, each interactive element gets assigned a ref ID
//! 2. The refs are deterministic within a snapshot (same page state = same refs)
//! 3. You can use refs directly in commands: `click("@e1")`, `type("@e2", "hello")`
//! 4. Refs are resolved to the actual element using stored locator information
//!
//! # Example
//!
//! ```text
//! Snapshot output:
//! [document] Example Page
//!   [heading level=1] Welcome
//!   [textbox @e1] Email
//!   [textbox @e2] Password
//!   [button @e3] Sign In
//!   [link @e4] Forgot password?
//!
//! Then use: click("@e3") to click the Sign In button
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// A reference ID like `@e1`, `@e2`, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RefId(String);

impl RefId {
    /// Create a new RefId from a number.
    pub fn new(n: usize) -> Self {
        Self(format!("e{}", n))
    }

    /// Get the ref string with @ prefix for display.
    pub fn display(&self) -> String {
        format!("@{}", self.0)
    }

    /// Get the internal ID without @ prefix.
    pub fn id(&self) -> &str {
        &self.0
    }

    /// Parse a ref from a string (with or without @ prefix).
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let id = s.strip_prefix('@').unwrap_or(s);

        // Validate format: should be 'e' followed by digits
        if id.starts_with('e') && id.len() > 1 && id[1..].chars().all(|c| c.is_ascii_digit()) {
            Some(Self(id.to_string()))
        } else {
            None
        }
    }

    /// Check if a string looks like a ref.
    pub fn is_ref(s: &str) -> bool {
        let s = s.trim();
        let Some(id) = s.strip_prefix('@') else {
            return false;
        };
        id.starts_with('e') && id.len() > 1 && id[1..].chars().all(|c| c.is_ascii_digit())
    }

    /// Get the numeric part of the ref.
    pub fn number(&self) -> Option<usize> {
        self.0[1..].parse().ok()
    }
}

impl fmt::Display for RefId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl FromStr for RefId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

/// Information stored for each ref to enable element location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefInfo {
    /// The accessibility role (button, textbox, link, etc.).
    pub role: String,

    /// The accessible name (text content, label, aria-label, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// CSS selector that can locate this element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// Index for nth-match when multiple elements have same role+name.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub nth: usize,

    /// Backend node ID from CDP (for direct element access).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_node_id: Option<i64>,

    /// XPath to the element (fallback locator).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xpath: Option<String>,

    /// Whether this element is interactive (clickable, editable, etc.).
    #[serde(default)]
    pub interactive: bool,

    /// Whether this element is currently visible.
    #[serde(default)]
    pub visible: bool,

    /// Element's bounding box (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<ElementBounds>,

    /// Value for input elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Checked state for checkboxes/radios.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,

    /// Whether the element is disabled.
    #[serde(default)]
    pub disabled: bool,

    /// Level for heading elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,

    /// Additional attributes that might be useful.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

impl RefInfo {
    /// Create a new RefInfo with minimal information.
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            name: None,
            selector: None,
            nth: 0,
            backend_node_id: None,
            xpath: None,
            interactive: false,
            visible: true,
            bounds: None,
            value: None,
            checked: None,
            disabled: false,
            level: None,
            attributes: HashMap::new(),
        }
    }

    /// Set the accessible name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the CSS selector.
    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
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

    /// Generate a Playwright-style locator string.
    pub fn to_locator_string(&self) -> String {
        // Prefer role-based locator
        if let Some(ref name) = self.name {
            if !name.is_empty() {
                return format!(
                    "role={}[name=\"{}\"]",
                    self.role,
                    escape_locator_value(name)
                );
            }
        }

        // Fall back to CSS selector
        if let Some(ref selector) = self.selector {
            return selector.clone();
        }

        // Fall back to role only (will match first)
        format!("role={}", self.role)
    }
}

/// Element bounding box.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementBounds {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

impl ElementBounds {
    /// Get the center point of the bounding box.
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if a point is inside the bounding box.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

/// Map from RefId to RefInfo for the current snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefMap {
    /// The refs in order of assignment.
    refs: Vec<RefId>,
    /// Map from RefId to RefInfo.
    info: HashMap<RefId, RefInfo>,
    /// Counter for generating new refs.
    counter: usize,
}

impl RefMap {
    /// Create a new empty RefMap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all refs (for new snapshot).
    pub fn clear(&mut self) {
        self.refs.clear();
        self.info.clear();
        self.counter = 0;
    }

    /// Add a new element and get its ref.
    pub fn add(&mut self, info: RefInfo) -> RefId {
        self.counter += 1;
        let ref_id = RefId::new(self.counter);
        self.refs.push(ref_id.clone());
        self.info.insert(ref_id.clone(), info);
        ref_id
    }

    /// Get info for a ref.
    pub fn get(&self, ref_id: &RefId) -> Option<&RefInfo> {
        self.info.get(ref_id)
    }

    /// Look up a ref by its string representation (with or without @).
    pub fn lookup(&self, ref_str: &str) -> Option<&RefInfo> {
        RefId::parse(ref_str).and_then(|id| self.info.get(&id))
    }

    /// Get all refs in order.
    pub fn refs(&self) -> &[RefId] {
        &self.refs
    }

    /// Get the number of refs.
    pub fn len(&self) -> usize {
        self.refs.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.refs.is_empty()
    }

    /// Iterate over all refs and their info.
    pub fn iter(&self) -> impl Iterator<Item = (&RefId, &RefInfo)> {
        self.refs
            .iter()
            .filter_map(|id| self.info.get(id).map(|info| (id, info)))
    }

    /// Find refs matching a predicate.
    pub fn find<F>(&self, predicate: F) -> Vec<(&RefId, &RefInfo)>
    where
        F: Fn(&RefInfo) -> bool,
    {
        self.iter().filter(|(_, info)| predicate(info)).collect()
    }

    /// Find refs by role.
    pub fn find_by_role(&self, role: &str) -> Vec<(&RefId, &RefInfo)> {
        self.find(|info| info.role.eq_ignore_ascii_case(role))
    }

    /// Find refs by name (partial match).
    pub fn find_by_name(&self, name: &str) -> Vec<(&RefId, &RefInfo)> {
        let name_lower = name.to_lowercase();
        self.find(|info| {
            info.name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&name_lower))
                .unwrap_or(false)
        })
    }

    /// Find interactive refs only.
    pub fn interactive(&self) -> Vec<(&RefId, &RefInfo)> {
        self.find(|info| info.interactive)
    }
}

/// Escape special characters in locator values.
fn escape_locator_value(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_id_parse() {
        assert!(RefId::parse("@e1").is_some());
        assert!(RefId::parse("e1").is_some());
        assert!(RefId::parse("@e123").is_some());
        assert!(RefId::parse("@e").is_none());
        assert!(RefId::parse("@a1").is_none());
        assert!(RefId::parse("button").is_none());
    }

    #[test]
    fn test_ref_id_is_ref() {
        assert!(RefId::is_ref("@e1"));
        assert!(RefId::is_ref("@e99"));
        assert!(!RefId::is_ref("e1"));
        assert!(!RefId::is_ref("button"));
        assert!(!RefId::is_ref("#submit"));
    }

    #[test]
    fn test_ref_map_add_and_lookup() {
        let mut map = RefMap::new();
        let info = RefInfo::new("button").with_name("Submit");
        let ref_id = map.add(info);

        assert_eq!(ref_id.display(), "@e1");
        assert!(map.lookup("@e1").is_some());
        assert!(map.lookup("e1").is_some());
        assert!(map.lookup("@e2").is_none());
    }

    #[test]
    fn test_ref_info_to_locator() {
        // With name - uses role-based locator
        let info = RefInfo::new("button").with_name("Submit");
        assert_eq!(info.to_locator_string(), "role=button[name=\"Submit\"]");

        // With CSS selector but no name - uses CSS selector
        let info2 = RefInfo::new("textbox").with_selector("input#email");
        assert_eq!(info2.to_locator_string(), "input#email");

        // With neither name nor selector - uses role only
        let info3 = RefInfo::new("textbox");
        assert_eq!(info3.to_locator_string(), "role=textbox");
    }
}
