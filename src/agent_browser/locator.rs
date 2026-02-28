//! Locator system for finding elements by various strategies.
//!
//! Supports:
//! - Refs (`@e1`, `@e2`) from snapshots
//! - Semantic locators (by role, text, label, placeholder, etc.)
//! - CSS selectors
//! - XPath expressions

use super::refs::{RefId, RefMap};
use super::response::AgentError;

/// Strategy for locating elements.
#[derive(Debug, Clone, PartialEq)]
pub enum LocatorStrategy {
    /// A ref from a snapshot (@e1, @e2, etc.).
    Ref(RefId),

    /// By accessibility role and optional name.
    Role {
        role: String,
        name: Option<String>,
        exact: bool,
    },

    /// By visible text content.
    Text {
        text: String,
        exact: bool,
    },

    /// By associated label text.
    Label {
        label: String,
        exact: bool,
    },

    /// By placeholder text.
    Placeholder {
        text: String,
        exact: bool,
    },

    /// By alt text (for images).
    AltText {
        text: String,
        exact: bool,
    },

    /// By title attribute.
    Title {
        title: String,
        exact: bool,
    },

    /// By data-testid attribute.
    TestId(String),

    /// CSS selector.
    Css(String),

    /// XPath expression.
    XPath(String),

    /// Nth element from another locator.
    Nth {
        base: Box<Locator>,
        index: i32, // -1 for last
    },
}

/// A locator that can find elements on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct Locator {
    /// The strategy to use for finding elements.
    pub strategy: LocatorStrategy,
}

impl Locator {
    /// Create a locator from a ref string.
    pub fn from_ref(ref_str: &str) -> Result<Self, AgentError> {
        RefId::parse(ref_str)
            .map(|id| Self {
                strategy: LocatorStrategy::Ref(id),
            })
            .ok_or_else(|| AgentError::InvalidRef {
                ref_str: ref_str.to_string(),
            })
    }

    /// Create a locator by role.
    pub fn by_role(role: impl Into<String>, name: Option<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Role {
                role: role.into(),
                name,
                exact: false,
            },
        }
    }

    /// Create a locator by role with exact name matching.
    pub fn by_role_exact(role: impl Into<String>, name: Option<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Role {
                role: role.into(),
                name,
                exact: true,
            },
        }
    }

    /// Create a locator by visible text.
    pub fn by_text(text: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Text {
                text: text.into(),
                exact: false,
            },
        }
    }

    /// Create a locator by exact visible text.
    pub fn by_text_exact(text: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Text {
                text: text.into(),
                exact: true,
            },
        }
    }

    /// Create a locator by label text.
    pub fn by_label(label: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Label {
                label: label.into(),
                exact: false,
            },
        }
    }

    /// Create a locator by exact label text.
    pub fn by_label_exact(label: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Label {
                label: label.into(),
                exact: true,
            },
        }
    }

    /// Create a locator by placeholder text.
    pub fn by_placeholder(text: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Placeholder {
                text: text.into(),
                exact: false,
            },
        }
    }

    /// Create a locator by exact placeholder text.
    pub fn by_placeholder_exact(text: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Placeholder {
                text: text.into(),
                exact: true,
            },
        }
    }

    /// Create a locator by alt text.
    pub fn by_alt_text(text: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::AltText {
                text: text.into(),
                exact: false,
            },
        }
    }

    /// Create a locator by exact alt text.
    pub fn by_alt_text_exact(text: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::AltText {
                text: text.into(),
                exact: true,
            },
        }
    }

    /// Create a locator by title attribute.
    pub fn by_title(title: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Title {
                title: title.into(),
                exact: false,
            },
        }
    }

    /// Create a locator by exact title attribute.
    pub fn by_title_exact(title: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Title {
                title: title.into(),
                exact: true,
            },
        }
    }

    /// Create a locator by data-testid.
    pub fn by_test_id(test_id: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::TestId(test_id.into()),
        }
    }

    /// Create a locator by CSS selector.
    pub fn css(selector: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::Css(selector.into()),
        }
    }

    /// Create a locator by XPath.
    pub fn xpath(expression: impl Into<String>) -> Self {
        Self {
            strategy: LocatorStrategy::XPath(expression.into()),
        }
    }

    /// Get the nth element (0-indexed, -1 for last).
    pub fn nth(self, index: i32) -> Self {
        Self {
            strategy: LocatorStrategy::Nth {
                base: Box::new(self),
                index,
            },
        }
    }

    /// Get the first element.
    pub fn first(self) -> Self {
        self.nth(0)
    }

    /// Get the last element.
    pub fn last(self) -> Self {
        self.nth(-1)
    }

    /// Check if this is a ref locator.
    pub fn is_ref(&self) -> bool {
        matches!(self.strategy, LocatorStrategy::Ref(_))
    }

    /// Get the ref ID if this is a ref locator.
    pub fn ref_id(&self) -> Option<&RefId> {
        match &self.strategy {
            LocatorStrategy::Ref(id) => Some(id),
            _ => None,
        }
    }

    /// Parse a selector string into a locator.
    ///
    /// Recognizes:
    /// - `@e1`, `@e2` etc. -> Ref
    /// - `role=button[name="Submit"]` -> Role
    /// - `text=Hello` -> Text
    /// - `label=Email` -> Label
    /// - `placeholder=Enter email` -> Placeholder
    /// - `[data-testid="submit"]` -> TestId
    /// - `//button` (starts with //) -> XPath
    /// - Anything else -> CSS selector
    pub fn parse(selector: &str) -> Self {
        let selector = selector.trim();

        // Check for ref
        if RefId::is_ref(selector) {
            if let Some(id) = RefId::parse(selector) {
                return Self {
                    strategy: LocatorStrategy::Ref(id),
                };
            }
        }

        // Check for role= prefix
        if let Some(rest) = selector.strip_prefix("role=") {
            return Self::parse_role_locator(rest);
        }

        // Check for text= prefix
        if let Some(rest) = selector.strip_prefix("text=") {
            let (text, exact) = parse_locator_value(rest);
            return Self {
                strategy: LocatorStrategy::Text { text, exact },
            };
        }

        // Check for label= prefix
        if let Some(rest) = selector.strip_prefix("label=") {
            let (label, exact) = parse_locator_value(rest);
            return Self {
                strategy: LocatorStrategy::Label { label, exact },
            };
        }

        // Check for placeholder= prefix
        if let Some(rest) = selector.strip_prefix("placeholder=") {
            let (text, exact) = parse_locator_value(rest);
            return Self {
                strategy: LocatorStrategy::Placeholder { text, exact },
            };
        }

        // Check for alt= prefix
        if let Some(rest) = selector.strip_prefix("alt=") {
            let (text, exact) = parse_locator_value(rest);
            return Self {
                strategy: LocatorStrategy::AltText { text, exact },
            };
        }

        // Check for title= prefix
        if let Some(rest) = selector.strip_prefix("title=") {
            let (title, exact) = parse_locator_value(rest);
            return Self {
                strategy: LocatorStrategy::Title { title, exact },
            };
        }

        // Check for data-testid selector
        if selector.starts_with("[data-testid=") {
            if let Some(test_id) = extract_attr_value(selector, "data-testid") {
                return Self::by_test_id(test_id);
            }
        }

        // Check for XPath (starts with // or /)
        if selector.starts_with("//") || selector.starts_with("xpath=") {
            let xpath = selector.strip_prefix("xpath=").unwrap_or(selector);
            return Self::xpath(xpath);
        }

        // Default to CSS selector
        Self::css(selector)
    }

    /// Parse a role= locator like "role=button[name=\"Submit\"]"
    fn parse_role_locator(s: &str) -> Self {
        // Format: role[name="value"] or just role
        if let Some(bracket_pos) = s.find('[') {
            let role = s[..bracket_pos].trim();
            let attrs = &s[bracket_pos..];

            // Look for name attribute
            let name = extract_attr_value(attrs, "name");
            let exact = attrs.contains("exact=true") || attrs.contains("exact=True");

            Self {
                strategy: LocatorStrategy::Role {
                    role: role.to_string(),
                    name,
                    exact,
                },
            }
        } else {
            Self {
                strategy: LocatorStrategy::Role {
                    role: s.trim().to_string(),
                    name: None,
                    exact: false,
                },
            }
        }
    }

    /// Convert to a CSS selector string (for use with existing Page methods).
    pub fn to_css_selector(&self, ref_map: Option<&RefMap>) -> Result<String, AgentError> {
        match &self.strategy {
            LocatorStrategy::Ref(ref_id) => {
                let ref_map = ref_map.ok_or(AgentError::NoSnapshot)?;
                let info = ref_map.get(ref_id).ok_or_else(|| AgentError::RefNotFound {
                    ref_id: ref_id.display(),
                })?;

                // Use CSS selector if available
                if let Some(ref selector) = info.selector {
                    return Ok(selector.clone());
                }

                // Generate from role + name
                Ok(self.role_to_css(&info.role, info.name.as_deref()))
            }

            LocatorStrategy::Role { role, name, .. } => Ok(self.role_to_css(role, name.as_deref())),

            LocatorStrategy::Text { text, exact } => {
                // Use :has-text() pseudo-selector (not standard CSS, for Playwright)
                // For standard CSS, we'd need JS evaluation
                if *exact {
                    Ok(format!(":text-is(\"{}\")", escape_css_string(text)))
                } else {
                    Ok(format!(":has-text(\"{}\")", escape_css_string(text)))
                }
            }

            LocatorStrategy::Label { label, .. } => {
                // Find by label's for attribute or aria-labelledby
                Ok(format!(
                    "[aria-label=\"{}\"], label:has-text(\"{}\") + input, label:has-text(\"{}\") input",
                    escape_css_string(label),
                    escape_css_string(label),
                    escape_css_string(label)
                ))
            }

            LocatorStrategy::Placeholder { text, exact } => {
                if *exact {
                    Ok(format!("[placeholder=\"{}\"]", escape_css_string(text)))
                } else {
                    Ok(format!("[placeholder*=\"{}\"]", escape_css_string(text)))
                }
            }

            LocatorStrategy::AltText { text, exact } => {
                if *exact {
                    Ok(format!("[alt=\"{}\"]", escape_css_string(text)))
                } else {
                    Ok(format!("[alt*=\"{}\"]", escape_css_string(text)))
                }
            }

            LocatorStrategy::Title { title, exact } => {
                if *exact {
                    Ok(format!("[title=\"{}\"]", escape_css_string(title)))
                } else {
                    Ok(format!("[title*=\"{}\"]", escape_css_string(title)))
                }
            }

            LocatorStrategy::TestId(test_id) => {
                Ok(format!("[data-testid=\"{}\"]", escape_css_string(test_id)))
            }

            LocatorStrategy::Css(selector) => Ok(selector.clone()),

            LocatorStrategy::XPath(_) => Err(AgentError::InvalidCommand {
                message: "XPath cannot be converted to CSS selector".to_string(),
            }),

            LocatorStrategy::Nth { base, index } => {
                let base_selector = base.to_css_selector(ref_map)?;
                // CSS :nth-of-type is 1-indexed
                if *index < 0 {
                    Ok(format!("{}:last-of-type", base_selector))
                } else {
                    Ok(format!("{}:nth-of-type({})", base_selector, index + 1))
                }
            }
        }
    }

    /// Convert ARIA role to approximate CSS selector.
    fn role_to_css(&self, role: &str, name: Option<&str>) -> String {
        let role_lower = role.to_lowercase();
        let base = match role_lower.as_str() {
            "button" => "button, [role=\"button\"], input[type=\"button\"], input[type=\"submit\"]",
            "link" => "a[href], [role=\"link\"]",
            "textbox" => "input:not([type]), input[type=\"text\"], input[type=\"email\"], input[type=\"password\"], input[type=\"search\"], input[type=\"tel\"], input[type=\"url\"], textarea, [role=\"textbox\"]",
            "searchbox" => "input[type=\"search\"], [role=\"searchbox\"]",
            "checkbox" => "input[type=\"checkbox\"], [role=\"checkbox\"]",
            "radio" => "input[type=\"radio\"], [role=\"radio\"]",
            "combobox" => "select, [role=\"combobox\"]",
            "listbox" => "select[multiple], [role=\"listbox\"]",
            "option" => "option, [role=\"option\"]",
            "heading" => "h1, h2, h3, h4, h5, h6, [role=\"heading\"]",
            "img" | "image" => "img, [role=\"img\"]",
            "navigation" => "nav, [role=\"navigation\"]",
            "main" => "main, [role=\"main\"]",
            "banner" => "header, [role=\"banner\"]",
            "contentinfo" => "footer, [role=\"contentinfo\"]",
            "form" => "form, [role=\"form\"]",
            "list" => "ul, ol, [role=\"list\"]",
            "listitem" => "li, [role=\"listitem\"]",
            "table" => "table, [role=\"table\"]",
            "row" => "tr, [role=\"row\"]",
            "cell" => "td, [role=\"cell\"]",
            "columnheader" => "th, [role=\"columnheader\"]",
            "dialog" => "dialog, [role=\"dialog\"]",
            "alert" => "[role=\"alert\"]",
            "tab" => "[role=\"tab\"]",
            "tabpanel" => "[role=\"tabpanel\"]",
            "menu" => "[role=\"menu\"]",
            "menuitem" => "[role=\"menuitem\"]",
            "slider" => "input[type=\"range\"], [role=\"slider\"]",
            "spinbutton" => "input[type=\"number\"], [role=\"spinbutton\"]",
            "switch" => "[role=\"switch\"]",
            "tree" => "[role=\"tree\"]",
            "treeitem" => "[role=\"treeitem\"]",
            "grid" => "[role=\"grid\"]",
            "gridcell" => "[role=\"gridcell\"]",
            _ => return format!("[role=\"{}\"]", role_lower),
        };

        // Add name filter if provided
        if let Some(name) = name {
            // This is a simplification - proper name matching requires aria-label, aria-labelledby, etc.
            format!(
                "{}:is([aria-label=\"{}\"], :has-text(\"{}\"))",
                base.split(", ").next().unwrap_or(base),
                escape_css_string(name),
                escape_css_string(name)
            )
        } else {
            base.to_string()
        }
    }

    /// Generate JavaScript code to find elements matching this locator.
    pub fn to_js_expression(&self, ref_map: Option<&RefMap>) -> Result<String, AgentError> {
        match &self.strategy {
            LocatorStrategy::Ref(ref_id) => {
                let ref_map = ref_map.ok_or(AgentError::NoSnapshot)?;
                let info = ref_map.get(ref_id).ok_or_else(|| AgentError::RefNotFound {
                    ref_id: ref_id.display(),
                })?;

                if let Some(backend_id) = info.backend_node_id {
                    // Use backend node ID directly if available
                    Ok(format!(
                        "(() => {{ throw new Error('Use CDP DOM.resolveNode for backendNodeId {}'); }})()",
                        backend_id
                    ))
                } else if let Some(ref selector) = info.selector {
                    Ok(format!("document.querySelector('{}')", escape_js_string(selector)))
                } else {
                    // Use role-based query
                    Ok(self.role_to_js(&info.role, info.name.as_deref(), info.nth))
                }
            }

            LocatorStrategy::Role { role, name, exact: _ } => Ok(self.role_to_js(role, name.as_deref(), 0)),

            LocatorStrategy::Text { text, exact } => {
                if *exact {
                    Ok(format!(
                        r#"[...document.querySelectorAll('*')].find(el => el.textContent?.trim() === '{}')"#,
                        escape_js_string(text)
                    ))
                } else {
                    Ok(format!(
                        r#"[...document.querySelectorAll('*')].find(el => el.textContent?.toLowerCase().includes('{}'))"#,
                        escape_js_string(&text.to_lowercase())
                    ))
                }
            }

            LocatorStrategy::Label { label, exact } => {
                if *exact {
                    Ok(format!(
                        r#"document.querySelector('[aria-label="{}"]') || document.querySelector('label')?.textContent?.trim() === '{}' && document.querySelector('label')?.control"#,
                        escape_js_string(label),
                        escape_js_string(label)
                    ))
                } else {
                    Ok(format!(
                        r#"document.querySelector('[aria-label*="{}"]') || [...document.querySelectorAll('label')].find(l => l.textContent?.toLowerCase().includes('{}'))?.control"#,
                        escape_js_string(label),
                        escape_js_string(&label.to_lowercase())
                    ))
                }
            }

            LocatorStrategy::Placeholder { text, exact } => {
                if *exact {
                    Ok(format!(
                        r#"document.querySelector('[placeholder="{}"]')"#,
                        escape_js_string(text)
                    ))
                } else {
                    Ok(format!(
                        r#"document.querySelector('[placeholder*="{}"]')"#,
                        escape_js_string(text)
                    ))
                }
            }

            LocatorStrategy::AltText { text, exact } => {
                if *exact {
                    Ok(format!(r#"document.querySelector('[alt="{}"]')"#, escape_js_string(text)))
                } else {
                    Ok(format!(r#"document.querySelector('[alt*="{}"]')"#, escape_js_string(text)))
                }
            }

            LocatorStrategy::Title { title, exact } => {
                if *exact {
                    Ok(format!(
                        r#"document.querySelector('[title="{}"]')"#,
                        escape_js_string(title)
                    ))
                } else {
                    Ok(format!(
                        r#"document.querySelector('[title*="{}"]')"#,
                        escape_js_string(title)
                    ))
                }
            }

            LocatorStrategy::TestId(test_id) => Ok(format!(
                r#"document.querySelector('[data-testid="{}"]')"#,
                escape_js_string(test_id)
            )),

            LocatorStrategy::Css(selector) => {
                Ok(format!("document.querySelector('{}')", escape_js_string(selector)))
            }

            LocatorStrategy::XPath(xpath) => Ok(format!(
                r#"document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue"#,
                escape_js_string(xpath)
            )),

            LocatorStrategy::Nth { base, index } => {
                let base_js = match &base.strategy {
                    LocatorStrategy::Css(s) => format!("document.querySelectorAll('{}')", escape_js_string(s)),
                    _ => return Err(AgentError::InvalidCommand {
                        message: "nth() only supported for CSS selectors in JS mode".to_string(),
                    }),
                };

                if *index < 0 {
                    Ok(format!("(() => {{ const els = {}; return els[els.length - 1]; }})()", base_js))
                } else {
                    Ok(format!("{}[{}]", base_js, index))
                }
            }
        }
    }

    /// Generate JS to find element by role and name.
    fn role_to_js(&self, role: &str, name: Option<&str>, nth: usize) -> String {
        let role_lower = role.to_lowercase();

        let name_check = if let Some(name) = name {
            format!(
                " && (el.getAttribute('aria-label')?.toLowerCase().includes('{}') || el.textContent?.toLowerCase().includes('{}'))",
                escape_js_string(&name.to_lowercase()),
                escape_js_string(&name.to_lowercase())
            )
        } else {
            String::new()
        };

        let nth_filter = if nth > 0 {
            format!("[{}]", nth)
        } else {
            String::new()
        };

        format!(
            r#"[...document.querySelectorAll('[role="{}"], {}')].filter(el => true{}){}"#,
            role_lower,
            role_to_native_elements(&role_lower),
            name_check,
            nth_filter
        )
    }
}

/// Get native HTML elements for an ARIA role.
fn role_to_native_elements(role: &str) -> &'static str {
    match role {
        "button" => "button, input[type='button'], input[type='submit']",
        "link" => "a[href]",
        "textbox" => "input:not([type]), input[type='text'], input[type='email'], input[type='password'], textarea",
        "checkbox" => "input[type='checkbox']",
        "radio" => "input[type='radio']",
        "combobox" | "listbox" => "select",
        "heading" => "h1, h2, h3, h4, h5, h6",
        "img" => "img",
        "list" => "ul, ol",
        "listitem" => "li",
        _ => "",
    }
}

/// Parse a locator value that might be quoted.
fn parse_locator_value(s: &str) -> (String, bool) {
    let s = s.trim();
    let exact = s.starts_with('"') && s.ends_with('"');
    let value = if exact {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    };
    (value, exact)
}

/// Extract attribute value from a selector string like [attr="value"].
fn extract_attr_value(s: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = s.find(&pattern) {
        let value_start = start + pattern.len();
        let rest = &s[value_start..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }

    // Try single quotes
    let pattern = format!("{}='", attr);
    if let Some(start) = s.find(&pattern) {
        let value_start = start + pattern.len();
        let rest = &s[value_start..];
        if let Some(end) = rest.find('\'') {
            return Some(rest[..end].to_string());
        }
    }

    None
}

/// Escape a string for use in CSS selectors.
fn escape_css_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a string for use in JavaScript.
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locator_parse_ref() {
        let loc = Locator::parse("@e1");
        assert!(matches!(loc.strategy, LocatorStrategy::Ref(_)));
    }

    #[test]
    fn test_locator_parse_role() {
        let loc = Locator::parse("role=button");
        assert!(matches!(
            loc.strategy,
            LocatorStrategy::Role { role, name: None, .. } if role == "button"
        ));

        let loc2 = Locator::parse("role=button[name=\"Submit\"]");
        assert!(matches!(
            loc2.strategy,
            LocatorStrategy::Role { role, name: Some(n), .. } if role == "button" && n == "Submit"
        ));
    }

    #[test]
    fn test_locator_parse_text() {
        let loc = Locator::parse("text=Hello World");
        assert!(matches!(
            loc.strategy,
            LocatorStrategy::Text { text, exact: false } if text == "Hello World"
        ));
    }

    #[test]
    fn test_locator_parse_css() {
        let loc = Locator::parse("#submit-btn");
        assert!(matches!(
            loc.strategy,
            LocatorStrategy::Css(s) if s == "#submit-btn"
        ));
    }

    #[test]
    fn test_locator_parse_xpath() {
        let loc = Locator::parse("//button[@id='submit']");
        assert!(matches!(
            loc.strategy,
            LocatorStrategy::XPath(s) if s == "//button[@id='submit']"
        ));
    }

    #[test]
    fn test_locator_parse_testid() {
        let loc = Locator::parse("[data-testid=\"submit-btn\"]");
        assert!(matches!(
            loc.strategy,
            LocatorStrategy::TestId(s) if s == "submit-btn"
        ));
    }
}
