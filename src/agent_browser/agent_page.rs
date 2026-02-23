//! AgentPage - High-level agent-browser compatible API.
//!
//! This module provides the main interface for AI agents to interact with web pages.
//! It wraps `ChaserPage` and adds the Snapshot + Refs system and semantic locators.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::chaser::ChaserPage;

use super::commands::{
    ClickOptions, ElementState, LoadState, MouseButton, ScreenshotFormat, ScreenshotOptions,
    ScrollDirection, TypeOptions, WaitOptions,
};
use super::locator::Locator;
use super::refs::{RefInfo, RefMap};
use super::response::*;
use super::snapshot::{is_interactive_role, AccessibilityNode, Snapshot, SnapshotOptions};

/// AgentPage provides an AI-agent friendly interface for browser automation.
///
/// It wraps `ChaserPage` and adds:
/// - Snapshot + Refs system for efficient element referencing
/// - Semantic locators (by role, text, label, etc.)
/// - JSON-serializable responses
/// - Cached ref map for element lookup
pub struct AgentPage {
    /// The underlying ChaserPage.
    chaser: ChaserPage,

    /// The current snapshot's ref map.
    ref_map: Arc<Mutex<RefMap>>,

    /// Console messages collected during the session.
    /// TODO: Wire up console tracking in a future phase.
    #[allow(dead_code)]
    console_messages: Arc<Mutex<Vec<ConsoleMessage>>>,

    /// Page errors collected during the session.
    /// TODO: Wire up error tracking in a future phase.
    #[allow(dead_code)]
    page_errors: Arc<Mutex<Vec<PageError>>>,

    /// Whether to track console messages.
    #[allow(dead_code)]
    track_console: bool,

    /// Whether to track page errors.
    #[allow(dead_code)]
    track_errors: bool,
}

impl AgentPage {
    /// Create a new AgentPage wrapping a ChaserPage.
    pub fn new(chaser: ChaserPage) -> Self {
        Self {
            chaser,
            ref_map: Arc::new(Mutex::new(RefMap::new())),
            console_messages: Arc::new(Mutex::new(Vec::new())),
            page_errors: Arc::new(Mutex::new(Vec::new())),
            track_console: true,
            track_errors: true,
        }
    }

    /// Get the underlying ChaserPage.
    pub fn chaser(&self) -> &ChaserPage {
        &self.chaser
    }

    /// Get mutable access to the underlying ChaserPage.
    pub fn chaser_mut(&mut self) -> &mut ChaserPage {
        &mut self.chaser
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Navigate to a URL.
    pub async fn navigate(&self, url: &str) -> AgentResult<NavigateData> {
        self.chaser.goto(url).await.map_err(|e| AgentError::Navigation {
            message: e.to_string(),
        })?;

        let current_url = self.get_url().await?;
        let title = self.get_title().await.ok();

        Ok(NavigateData {
            url: current_url,
            title,
        })
    }

    /// Alias for navigate.
    pub async fn open(&self, url: &str) -> AgentResult<NavigateData> {
        self.navigate(url).await
    }

    /// Alias for navigate.
    pub async fn goto(&self, url: &str) -> AgentResult<NavigateData> {
        self.navigate(url).await
    }

    /// Go back in history.
    pub async fn back(&self) -> AgentResult<()> {
        self.chaser
            .evaluate("window.history.back()")
            .await
            .map_err(|e| AgentError::Navigation {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Go forward in history.
    pub async fn forward(&self) -> AgentResult<()> {
        self.chaser
            .evaluate("window.history.forward()")
            .await
            .map_err(|e| AgentError::Navigation {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Reload the page.
    pub async fn reload(&self) -> AgentResult<()> {
        self.chaser
            .evaluate("window.location.reload()")
            .await
            .map_err(|e| AgentError::Navigation {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Close the page.
    ///
    /// Note: This consumes the AgentPage since the underlying page is closed.
    pub async fn close(self) -> AgentResult<()> {
        self.chaser
            .into_raw_page()
            .close()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to close page: {}", e),
            })?;
        Ok(())
    }

    // =========================================================================
    // Snapshot & Refs
    // =========================================================================

    /// Take a snapshot of the page's accessibility tree.
    ///
    /// This is the core innovation of agent-browser - it returns a streamlined
    /// accessibility tree where each interactive element has a ref (`@e1`, `@e2`, etc.)
    /// that can be used in subsequent commands.
    pub async fn snapshot(&self, options: SnapshotOptions) -> AgentResult<Snapshot> {
        // Use JavaScript to build an accessibility-like tree
        // This is more reliable than the CDP Accessibility API for cross-browser compatibility
        let js = r#"
        (function() {
            const INTERACTIVE_ROLES = ['button', 'link', 'textbox', 'searchbox', 'checkbox', 'radio',
                'combobox', 'listbox', 'option', 'menuitem', 'tab', 'slider', 'spinbutton', 'switch'];

            function getRole(el) {
                // Check explicit role
                const explicitRole = el.getAttribute('role');
                if (explicitRole) return explicitRole;

                // Infer from tag
                const tag = el.tagName.toLowerCase();
                const type = el.type?.toLowerCase();

                switch(tag) {
                    case 'a': return el.href ? 'link' : 'generic';
                    case 'button': return 'button';
                    case 'input':
                        switch(type) {
                            case 'button': case 'submit': case 'reset': return 'button';
                            case 'checkbox': return 'checkbox';
                            case 'radio': return 'radio';
                            case 'range': return 'slider';
                            case 'number': return 'spinbutton';
                            case 'search': return 'searchbox';
                            default: return 'textbox';
                        }
                    case 'select': return el.multiple ? 'listbox' : 'combobox';
                    case 'option': return 'option';
                    case 'textarea': return 'textbox';
                    case 'img': return 'img';
                    case 'h1': case 'h2': case 'h3': case 'h4': case 'h5': case 'h6': return 'heading';
                    case 'nav': return 'navigation';
                    case 'main': return 'main';
                    case 'header': return 'banner';
                    case 'footer': return 'contentinfo';
                    case 'form': return 'form';
                    case 'ul': case 'ol': return 'list';
                    case 'li': return 'listitem';
                    case 'table': return 'table';
                    case 'tr': return 'row';
                    case 'td': return 'cell';
                    case 'th': return 'columnheader';
                    case 'dialog': return 'dialog';
                    default: return 'generic';
                }
            }

            function getName(el) {
                // aria-label
                const ariaLabel = el.getAttribute('aria-label');
                if (ariaLabel) return ariaLabel;

                // aria-labelledby
                const labelledBy = el.getAttribute('aria-labelledby');
                if (labelledBy) {
                    const labelEl = document.getElementById(labelledBy);
                    if (labelEl) return labelEl.textContent?.trim();
                }

                // For inputs, check associated label
                if (el.id) {
                    const label = document.querySelector(`label[for="${el.id}"]`);
                    if (label) return label.textContent?.trim();
                }

                // Placeholder for inputs
                if (el.placeholder) return el.placeholder;

                // Alt text for images
                if (el.alt) return el.alt;

                // Title attribute
                if (el.title) return el.title;

                // Text content (for buttons, links, etc.)
                const text = el.textContent?.trim();
                if (text && text.length < 100) return text;

                return null;
            }

            function getValue(el) {
                if (el.value !== undefined && el.value !== '') return el.value;
                return null;
            }

            function isInteractive(role) {
                return INTERACTIVE_ROLES.includes(role);
            }

            function isVisible(el) {
                const style = window.getComputedStyle(el);
                return style.display !== 'none' &&
                       style.visibility !== 'hidden' &&
                       style.opacity !== '0' &&
                       el.offsetWidth > 0 &&
                       el.offsetHeight > 0;
            }

            function buildTree(el, depth = 0, maxDepth = 20) {
                if (depth > maxDepth) return null;
                if (!el || el.nodeType !== 1) return null;

                const role = getRole(el);
                const name = getName(el);
                const visible = isVisible(el);
                const interactive = isInteractive(role);

                const node = {
                    role,
                    name,
                    visible,
                    interactive,
                    value: getValue(el),
                    checked: el.checked,
                    disabled: el.disabled,
                    children: []
                };

                // Add level for headings
                if (role === 'heading') {
                    const tag = el.tagName.toLowerCase();
                    node.level = parseInt(tag.charAt(1)) || 1;
                }

                // Build children
                for (const child of el.children) {
                    const childNode = buildTree(child, depth + 1, maxDepth);
                    if (childNode) {
                        node.children.push(childNode);
                    }
                }

                return node;
            }

            return JSON.stringify(buildTree(document.body));
        })()
        "#;

        let result = self.chaser.evaluate(js).await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get accessibility tree: {}", e),
        })?;

        let tree_json = result
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| AgentError::Internal {
                message: "Failed to parse accessibility tree".to_string(),
            })?;

        // Parse the JSON tree
        let js_tree: serde_json::Value = serde_json::from_str(&tree_json).map_err(|e| AgentError::Internal {
            message: format!("Failed to parse tree JSON: {}", e),
        })?;

        // Convert to our AccessibilityNode structure
        let root = self.convert_js_tree(&js_tree, &options, 0)?;

        // Create the snapshot
        let mut snapshot = Snapshot::from_tree(root, &options);

        // Add page metadata
        if let Ok(url) = self.get_url().await {
            snapshot = snapshot.with_url(url);
        }
        if let Ok(title) = self.get_title().await {
            snapshot = snapshot.with_title(title);
        }

        // Update the cached ref map
        {
            let mut ref_map = self.ref_map.lock().await;
            *ref_map = snapshot.refs.clone();
        }

        Ok(snapshot)
    }

    /// Convert JavaScript tree to AccessibilityNode.
    fn convert_js_tree(
        &self,
        value: &serde_json::Value,
        options: &SnapshotOptions,
        depth: usize,
    ) -> AgentResult<AccessibilityNode> {
        let role = value["role"].as_str().unwrap_or("generic").to_string();
        let name = value["name"].as_str().map(String::from);
        let visible = value["visible"].as_bool().unwrap_or(true);
        let interactive = value["interactive"].as_bool().unwrap_or(false);

        // Check depth limit
        if options.max_depth > 0 && depth >= options.max_depth {
            return Ok(AccessibilityNode::new(&role));
        }

        // Skip hidden elements unless requested
        if !visible && !options.include_hidden {
            return Ok(AccessibilityNode::new("generic")); // Return placeholder
        }

        let mut node = AccessibilityNode::new(&role);
        if let Some(n) = name {
            node = node.with_name(n);
        }

        node.visible = visible;
        node.interactive = interactive;
        node.depth = depth;

        // Extract value
        if let Some(val) = value["value"].as_str() {
            node.value = Some(val.to_string());
        }

        // Extract checked state
        if let Some(checked) = value["checked"].as_bool() {
            node.checked = Some(checked);
        }

        // Extract disabled state
        if let Some(disabled) = value["disabled"].as_bool() {
            node.disabled = disabled;
        }

        // Extract level for headings
        if let Some(level) = value["level"].as_i64() {
            node.level = Some(level as u8);
        }

        // Mark as interactive based on role
        if is_interactive_role(&role) {
            node = node.interactive();
        }

        // Process children
        if let Some(children) = value["children"].as_array() {
            for child in children {
                if let Ok(child_node) = self.convert_js_tree(child, options, depth + 1) {
                    // Filter based on options
                    if options.interactive_only && !child_node.interactive && child_node.children.is_empty() {
                        continue;
                    }
                    node.children.push(child_node);
                }
            }
        }

        Ok(node)
    }

    /// Get the current ref map.
    pub async fn get_ref_map(&self) -> RefMap {
        self.ref_map.lock().await.clone()
    }

    /// Look up a ref in the current snapshot.
    pub async fn lookup_ref(&self, ref_str: &str) -> AgentResult<RefInfo> {
        let ref_map = self.ref_map.lock().await;
        ref_map
            .lookup(ref_str)
            .cloned()
            .ok_or_else(|| AgentError::RefNotFound {
                ref_id: ref_str.to_string(),
            })
    }

    // =========================================================================
    // Element Actions
    // =========================================================================

    /// Click an element by selector or ref.
    pub async fn click(&self, selector: &str) -> AgentResult<()> {
        self.click_with_options(selector, ClickOptions::default()).await
    }

    /// Click an element with options.
    pub async fn click_with_options(&self, selector: &str, options: ClickOptions) -> AgentResult<()> {
        let element = self.find_element(selector).await?;

        if options.human_like {
            // Use ChaserPage's human-like click
            match element.bounding_box().await {
                Ok(bbox) => {
                    let center_x = bbox.x + bbox.width / 2.0;
                    let center_y = bbox.y + bbox.height / 2.0;
                    self.chaser.click_human(center_x, center_y).await.map_err(|e| {
                        AgentError::Internal {
                            message: format!("Human click failed: {}", e),
                        }
                    })?;
                }
                Err(_) => {
                    element.click().await.map_err(|e| AgentError::Internal {
                        message: format!("Click failed: {}", e),
                    })?;
                }
            }
        } else {
            for _ in 0..options.click_count {
                element.click().await.map_err(|e| AgentError::Internal {
                    message: format!("Click failed: {}", e),
                })?;
                if options.delay > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(options.delay)).await;
                }
            }
        }

        Ok(())
    }

    /// Double-click an element.
    pub async fn dblclick(&self, selector: &str) -> AgentResult<()> {
        self.click_with_options(
            selector,
            ClickOptions {
                click_count: 2,
                ..Default::default()
            },
        )
        .await
    }

    /// Hover over an element.
    pub async fn hover(&self, selector: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        element.hover().await.map_err(|e| AgentError::Internal {
            message: format!("Hover failed: {}", e),
        })?;
        Ok(())
    }

    /// Focus an element.
    pub async fn focus(&self, selector: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        element.focus().await.map_err(|e| AgentError::Internal {
            message: format!("Focus failed: {}", e),
        })?;
        Ok(())
    }

    /// Type text into an element.
    pub async fn type_text(&self, selector: &str, text: &str) -> AgentResult<()> {
        self.type_with_options(selector, text, TypeOptions::default()).await
    }

    /// Type text with options.
    pub async fn type_with_options(
        &self,
        selector: &str,
        text: &str,
        options: TypeOptions,
    ) -> AgentResult<()> {
        let element = self.find_element(selector).await?;

        if options.clear {
            element.focus().await.map_err(|e| AgentError::Internal {
                message: format!("Focus failed: {}", e),
            })?;
            self.chaser
                .evaluate("document.execCommand('selectAll', false, null)")
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Select all failed: {}", e),
                })?;
            self.chaser
                .evaluate("document.execCommand('delete', false, null)")
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Delete failed: {}", e),
                })?;
        }

        if options.human_like || options.with_typos {
            element.focus().await.map_err(|e| AgentError::Internal {
                message: format!("Focus failed: {}", e),
            })?;
            if options.with_typos {
                self.chaser.type_text_with_typos(text).await.map_err(|e| AgentError::Internal {
                    message: format!("Type failed: {}", e),
                })?;
            } else {
                self.chaser.type_text(text).await.map_err(|e| AgentError::Internal {
                    message: format!("Type failed: {}", e),
                })?;
            }
        } else {
            element
                .type_str(text)
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Type failed: {}", e),
                })?;
        }

        Ok(())
    }

    /// Fill an input with a value (clears first).
    pub async fn fill(&self, selector: &str, value: &str) -> AgentResult<()> {
        self.type_with_options(
            selector,
            value,
            TypeOptions {
                clear: true,
                ..Default::default()
            },
        )
        .await
    }

    /// Clear an input field.
    pub async fn clear(&self, selector: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        element.focus().await.map_err(|e| AgentError::Internal {
            message: format!("Focus failed: {}", e),
        })?;
        self.chaser
            .evaluate("document.execCommand('selectAll', false, null)")
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Select all failed: {}", e),
            })?;
        self.chaser
            .evaluate("document.execCommand('delete', false, null)")
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Delete failed: {}", e),
            })?;
        Ok(())
    }

    /// Check a checkbox or radio button.
    pub async fn check(&self, selector: &str) -> AgentResult<()> {
        let is_checked = self.is_checked(selector).await?;
        if !is_checked {
            let element = self.find_element(selector).await?;
            element.click().await.map_err(|e| AgentError::Internal {
                message: format!("Click failed: {}", e),
            })?;
        }
        Ok(())
    }

    /// Uncheck a checkbox.
    pub async fn uncheck(&self, selector: &str) -> AgentResult<()> {
        let is_checked = self.is_checked(selector).await?;
        if is_checked {
            let element = self.find_element(selector).await?;
            element.click().await.map_err(|e| AgentError::Internal {
                message: format!("Click failed: {}", e),
            })?;
        }
        Ok(())
    }

    /// Select an option in a dropdown.
    pub async fn select(&self, selector: &str, value: &str) -> AgentResult<()> {
        let js = format!(
            r#"
            const select = document.querySelector('{}');
            if (select) {{
                select.value = '{}';
                select.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }}
            "#,
            selector.replace('\'', "\\'"),
            value.replace('\'', "\\'")
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(())
    }

    /// Select multiple options in a multi-select dropdown.
    pub async fn multi_select(&self, selector: &str, values: &[&str]) -> AgentResult<()> {
        let values_json = serde_json::to_string(values).map_err(|e| AgentError::Internal {
            message: format!("Failed to serialize values: {}", e),
        })?;
        let js = format!(
            r#"
            const select = document.querySelector('{}');
            if (select && select.multiple) {{
                const values = {};
                for (const option of select.options) {{
                    option.selected = values.includes(option.value);
                }}
                select.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }}
            "#,
            selector.replace('\'', "\\'"),
            values_json
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(())
    }

    /// Select all text in an element.
    pub async fn select_all(&self, selector: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        element.focus().await.map_err(|e| AgentError::Internal {
            message: format!("Focus failed: {}", e),
        })?;
        self.chaser
            .evaluate("document.execCommand('selectAll', false, null)")
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Select all failed: {}", e),
            })?;
        Ok(())
    }

    /// Upload files to a file input.
    pub async fn upload(&self, selector: &str, files: &[&str]) -> AgentResult<()> {
        use crate::cdp::browser_protocol::dom::{SetFileInputFilesParams};

        let element = self.find_element(selector).await?;

        // Get the backend node ID from the element
        let node = element.description().await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get node description: {}", e),
        })?;

        let backend_node_id = node.backend_node_id;

        // Use CDP to set file input files
        let files_vec: Vec<String> = files.iter().map(|s| s.to_string()).collect();
        let params = SetFileInputFilesParams::builder()
            .files(files_vec)
            .backend_node_id(backend_node_id)
            .build()
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to build params: {}", e),
            })?;

        self.chaser
            .raw_page()
            .execute(params)
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Upload failed: {}", e),
            })?;
        Ok(())
    }

    /// Tap an element (touch interaction).
    pub async fn tap(&self, selector: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        match element.bounding_box().await {
            Ok(bbox) => {
                let center_x = bbox.x + bbox.width / 2.0;
                let center_y = bbox.y + bbox.height / 2.0;

                // Dispatch touch events via JavaScript
                let js = format!(
                    r#"
                    const el = document.querySelector('{}');
                    if (el) {{
                        const touch = new Touch({{
                            identifier: 1,
                            target: el,
                            clientX: {},
                            clientY: {},
                            pageX: {},
                            pageY: {}
                        }});
                        el.dispatchEvent(new TouchEvent('touchstart', {{
                            touches: [touch],
                            targetTouches: [touch],
                            changedTouches: [touch],
                            bubbles: true
                        }}));
                        el.dispatchEvent(new TouchEvent('touchend', {{
                            touches: [],
                            targetTouches: [],
                            changedTouches: [touch],
                            bubbles: true
                        }}));
                    }}
                    "#,
                    selector.replace('\'', "\\'"),
                    center_x,
                    center_y,
                    center_x,
                    center_y
                );
                self.chaser.evaluate(&js).await.map_err(|e| AgentError::Internal {
                    message: format!("Tap failed: {}", e),
                })?;
            }
            Err(_) => {
                // Fallback to click
                element.click().await.map_err(|e| AgentError::Internal {
                    message: format!("Tap (click fallback) failed: {}", e),
                })?;
            }
        }
        Ok(())
    }

    /// Drag an element to another element.
    pub async fn drag(&self, source: &str, target: &str) -> AgentResult<()> {
        let source_el = self.find_element(source).await?;
        let target_el = self.find_element(target).await?;

        let source_box = source_el.bounding_box().await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get source bounding box: {}", e),
        })?;
        let target_box = target_el.bounding_box().await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get target bounding box: {}", e),
        })?;

        let source_x = source_box.x + source_box.width / 2.0;
        let source_y = source_box.y + source_box.height / 2.0;
        let target_x = target_box.x + target_box.width / 2.0;
        let target_y = target_box.y + target_box.height / 2.0;

        // Use human-like mouse movement for drag
        self.chaser.move_mouse_human(source_x, source_y).await.map_err(|e| {
            AgentError::Internal {
                message: format!("Mouse move failed: {}", e),
            }
        })?;

        // Mouse down
        self.mouse_down(MouseButton::Left).await?;

        // Move to target
        self.chaser.move_mouse_human(target_x, target_y).await.map_err(|e| {
            AgentError::Internal {
                message: format!("Mouse move failed: {}", e),
            }
        })?;

        // Mouse up
        self.mouse_up(MouseButton::Left).await?;

        Ok(())
    }

    /// Dispatch a custom event on an element.
    pub async fn dispatch_event(
        &self,
        selector: &str,
        event_type: &str,
        event_init: Option<&serde_json::Value>,
    ) -> AgentResult<()> {
        let init_json = event_init
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string());

        let js = format!(
            r#"
            const el = document.querySelector('{}');
            if (el) {{
                const event = new Event('{}', {});
                el.dispatchEvent(event);
            }}
            "#,
            selector.replace('\'', "\\'"),
            event_type,
            init_json
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(())
    }

    /// Highlight an element (for debugging).
    pub async fn highlight(&self, selector: &str) -> AgentResult<()> {
        let js = format!(
            r#"
            const el = document.querySelector('{}');
            if (el) {{
                el.style.outline = '3px solid red';
                el.style.outlineOffset = '2px';
            }}
            "#,
            selector.replace('\'', "\\'")
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(())
    }

    /// Set a value on an element (alias for fill).
    pub async fn set_value(&self, selector: &str, value: &str) -> AgentResult<()> {
        self.fill(selector, value).await
    }

    // =========================================================================
    // Semantic Locator Actions
    // =========================================================================

    /// Click an element by role and optional name.
    pub async fn click_by_role(&self, role: &str, name: Option<&str>) -> AgentResult<()> {
        let locator = Locator::by_role(role, name.map(String::from));
        let selector = self.locator_to_selector(&locator).await?;
        self.click(&selector).await
    }

    /// Click an element by text content.
    pub async fn click_by_text(&self, text: &str) -> AgentResult<()> {
        let locator = Locator::by_text(text);
        let selector = self.locator_to_selector(&locator).await?;
        self.click(&selector).await
    }

    /// Click an element by label.
    pub async fn click_by_label(&self, label: &str) -> AgentResult<()> {
        let locator = Locator::by_label(label);
        let selector = self.locator_to_selector(&locator).await?;
        self.click(&selector).await
    }

    /// Fill an input by its label.
    pub async fn fill_by_label(&self, label: &str, value: &str) -> AgentResult<()> {
        let locator = Locator::by_label(label);
        let selector = self.locator_to_selector(&locator).await?;
        self.fill(&selector, value).await
    }

    /// Fill an input by its placeholder.
    pub async fn fill_by_placeholder(&self, placeholder: &str, value: &str) -> AgentResult<()> {
        let locator = Locator::by_placeholder(placeholder);
        let selector = self.locator_to_selector(&locator).await?;
        self.fill(&selector, value).await
    }

    // =========================================================================
    // Information Retrieval
    // =========================================================================

    /// Get the current URL.
    pub async fn get_url(&self) -> AgentResult<String> {
        self.chaser
            .url()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to get URL: {}", e),
            })?
            .ok_or_else(|| AgentError::Internal {
                message: "No URL available".to_string(),
            })
    }

    /// Get the page title.
    pub async fn get_title(&self) -> AgentResult<String> {
        let result = self
            .chaser
            .evaluate("document.title")
            .await
            .map_err(|e| AgentError::JavaScript {
                message: e.to_string(),
            })?;
        result
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| AgentError::Internal {
                message: "Failed to get title".to_string(),
            })
    }

    /// Get text content of an element.
    pub async fn get_text(&self, selector: &str) -> AgentResult<String> {
        let element = self.find_element(selector).await?;
        element
            .inner_text()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to get text: {}", e),
            })?
            .ok_or_else(|| AgentError::Internal {
                message: "No text content".to_string(),
            })
    }

    /// Get HTML content of an element.
    pub async fn get_html(&self, selector: Option<&str>) -> AgentResult<String> {
        if let Some(sel) = selector {
            let element = self.find_element(sel).await?;
            element
                .inner_html()
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Failed to get HTML: {}", e),
                })
                .map(|opt| opt.unwrap_or_default())
        } else {
            self.chaser.content().await.map_err(|e| AgentError::Internal {
                message: format!("Failed to get page HTML: {}", e),
            })
        }
    }

    /// Get the full page content (HTML).
    pub async fn content(&self) -> AgentResult<String> {
        self.chaser.content().await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get page content: {}", e),
        })
    }

    /// Get inner text of an element (alias for get_text).
    pub async fn inner_text(&self, selector: &str) -> AgentResult<String> {
        self.get_text(selector).await
    }

    /// Get inner HTML of an element.
    pub async fn inner_html(&self, selector: &str) -> AgentResult<String> {
        self.get_html(Some(selector)).await
    }

    /// Get the value of an input element.
    pub async fn get_value(&self, selector: &str) -> AgentResult<String> {
        let element = self.find_element(selector).await?;
        element
            .property("value")
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to get value: {}", e),
            })?
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| AgentError::Internal {
                message: "No value".to_string(),
            })
    }

    /// Get an attribute value.
    pub async fn get_attribute(&self, selector: &str, attribute: &str) -> AgentResult<Option<String>> {
        let element = self.find_element(selector).await?;
        element.attribute(attribute).await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get attribute: {}", e),
        })
    }

    /// Count elements matching a selector.
    pub async fn get_count(&self, selector: &str) -> AgentResult<usize> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;
        let js = format!(
            "document.querySelectorAll('{}').length",
            css.replace('\'', "\\'")
        );
        let result = self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        result
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .ok_or_else(|| AgentError::Internal {
                message: "Failed to count elements".to_string(),
            })
    }

    /// Get bounding box of an element.
    pub async fn get_bounding_box(&self, selector: &str) -> AgentResult<BoundingBoxData> {
        let element = self.find_element(selector).await?;
        let bbox = element.bounding_box().await.map_err(|e| AgentError::Internal {
            message: format!("Failed to get bounding box: {}", e),
        })?;

        Ok(BoundingBoxData {
            x: bbox.x,
            y: bbox.y,
            width: bbox.width,
            height: bbox.height,
        })
    }

    /// Get computed styles of an element.
    pub async fn get_styles(&self, selector: &str) -> AgentResult<serde_json::Value> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;
        let js = format!(
            r#"
            const el = document.querySelector('{}');
            if (!el) return null;
            const styles = window.getComputedStyle(el);
            const rect = el.getBoundingClientRect();
            return {{
                display: styles.display,
                visibility: styles.visibility,
                opacity: styles.opacity,
                position: styles.position,
                width: styles.width,
                height: styles.height,
                color: styles.color,
                backgroundColor: styles.backgroundColor,
                fontSize: styles.fontSize,
                fontWeight: styles.fontWeight,
                fontFamily: styles.fontFamily,
                margin: styles.margin,
                padding: styles.padding,
                border: styles.border,
                boxSizing: styles.boxSizing,
                boundingBox: {{
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    top: rect.top,
                    right: rect.right,
                    bottom: rect.bottom,
                    left: rect.left
                }}
            }};
            "#,
            css.replace('\'', "\\'")
        );
        let result = self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    // =========================================================================
    // State Checking
    // =========================================================================

    /// Check if an element is visible.
    pub async fn is_visible(&self, selector: &str) -> AgentResult<bool> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;
        let js = format!(
            r#"
            const el = document.querySelector('{}');
            if (!el) return false;
            const style = window.getComputedStyle(el);
            return style.display !== 'none' &&
                   style.visibility !== 'hidden' &&
                   style.opacity !== '0' &&
                   el.offsetWidth > 0 &&
                   el.offsetHeight > 0;
            "#,
            css.replace('\'', "\\'")
        );
        let result = self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(result.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Check if an element is enabled.
    pub async fn is_enabled(&self, selector: &str) -> AgentResult<bool> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;
        let js = format!(
            r#"
            const el = document.querySelector('{}');
            return el ? !el.disabled : false;
            "#,
            css.replace('\'', "\\'")
        );
        let result = self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(result.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Check if a checkbox/radio is checked.
    pub async fn is_checked(&self, selector: &str) -> AgentResult<bool> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;
        let js = format!(
            r#"
            const el = document.querySelector('{}');
            return el ? el.checked : false;
            "#,
            css.replace('\'', "\\'")
        );
        let result = self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(result.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Check if an element is editable.
    pub async fn is_editable(&self, selector: &str) -> AgentResult<bool> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;
        let js = format!(
            r#"
            const el = document.querySelector('{}');
            if (!el) return false;
            const tag = el.tagName.toLowerCase();
            if (tag === 'input' || tag === 'textarea') {{
                return !el.disabled && !el.readOnly;
            }}
            return el.isContentEditable;
            "#,
            css.replace('\'', "\\'")
        );
        let result = self.chaser.evaluate(&js).await.map_err(|e| AgentError::JavaScript {
            message: e.to_string(),
        })?;
        Ok(result.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Check if an element is hidden.
    pub async fn is_hidden(&self, selector: &str) -> AgentResult<bool> {
        let visible = self.is_visible(selector).await?;
        Ok(!visible)
    }

    // =========================================================================
    // Keyboard & Mouse
    // =========================================================================

    /// Press a key.
    pub async fn press(&self, key: &str) -> AgentResult<()> {
        let js = format!(
            r#"
            document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {{ key: '{}', bubbles: true }}));
            document.activeElement.dispatchEvent(new KeyboardEvent('keyup', {{ key: '{}', bubbles: true }}));
            "#,
            key, key
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::Internal {
            message: format!("Press key failed: {}", e),
        })?;
        Ok(())
    }

    /// Press a key on a specific element.
    pub async fn press_on(&self, selector: &str, key: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        element.focus().await.map_err(|e| AgentError::Internal {
            message: format!("Focus failed: {}", e),
        })?;
        self.press(key).await
    }

    /// Execute a keyboard shortcut (e.g., "Control+a", "Shift+Enter").
    pub async fn keyboard(&self, shortcut: &str) -> AgentResult<()> {
        // Parse the shortcut into modifiers and key
        let parts: Vec<&str> = shortcut.split('+').collect();
        let key = parts.last().ok_or_else(|| AgentError::Internal {
            message: "Invalid shortcut".to_string(),
        })?;

        let mut modifiers = Vec::new();
        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "control" | "ctrl" => modifiers.push("ctrlKey: true"),
                "shift" => modifiers.push("shiftKey: true"),
                "alt" => modifiers.push("altKey: true"),
                "meta" | "cmd" | "command" => modifiers.push("metaKey: true"),
                _ => {}
            }
        }

        let modifiers_str = modifiers.join(", ");
        let js = format!(
            r#"
            const event = new KeyboardEvent('keydown', {{
                key: '{}',
                code: '{}',
                {},
                bubbles: true
            }});
            document.activeElement.dispatchEvent(event);
            document.activeElement.dispatchEvent(new KeyboardEvent('keyup', {{
                key: '{}',
                code: '{}',
                {},
                bubbles: true
            }}));
            "#,
            key, key, modifiers_str, key, key, modifiers_str
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::Internal {
            message: format!("Keyboard shortcut failed: {}", e),
        })?;
        Ok(())
    }

    /// Hold a key down.
    pub async fn key_down(&self, key: &str) -> AgentResult<()> {
        let js = format!(
            r#"
            document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {{
                key: '{}',
                code: '{}',
                bubbles: true,
                repeat: false
            }}));
            "#,
            key, key
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::Internal {
            message: format!("Key down failed: {}", e),
        })?;
        Ok(())
    }

    /// Release a key.
    pub async fn key_up(&self, key: &str) -> AgentResult<()> {
        let js = format!(
            r#"
            document.activeElement.dispatchEvent(new KeyboardEvent('keyup', {{
                key: '{}',
                code: '{}',
                bubbles: true
            }}));
            "#,
            key, key
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::Internal {
            message: format!("Key up failed: {}", e),
        })?;
        Ok(())
    }

    /// Insert text without key events.
    pub async fn insert_text(&self, text: &str) -> AgentResult<()> {
        let js = format!(
            r#"
            const el = document.activeElement;
            if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) {{
                document.execCommand('insertText', false, '{}');
            }}
            "#,
            text.replace('\'', "\\'").replace('\n', "\\n")
        );
        self.chaser.evaluate(&js).await.map_err(|e| AgentError::Internal {
            message: format!("Insert text failed: {}", e),
        })?;
        Ok(())
    }

    /// Move mouse to coordinates.
    pub async fn mouse_move(&self, x: f64, y: f64) -> AgentResult<()> {
        self.chaser
            .move_mouse_human(x, y)
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Mouse move failed: {}", e),
            })?;
        Ok(())
    }

    /// Press mouse button down.
    pub async fn mouse_down(&self, button: MouseButton) -> AgentResult<()> {
        use crate::cdp::browser_protocol::input::{
            DispatchMouseEventParams, DispatchMouseEventType, MouseButton as CdpMouseButton,
        };

        let cdp_button = match button {
            MouseButton::Left => CdpMouseButton::Left,
            MouseButton::Middle => CdpMouseButton::Middle,
            MouseButton::Right => CdpMouseButton::Right,
        };

        let params = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MousePressed)
            .x(0.0)
            .y(0.0)
            .button(cdp_button)
            .click_count(1)
            .build()
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to build params: {}", e),
            })?;

        self.chaser
            .raw_page()
            .execute(params)
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Mouse down failed: {}", e),
            })?;
        Ok(())
    }

    /// Release mouse button.
    pub async fn mouse_up(&self, button: MouseButton) -> AgentResult<()> {
        use crate::cdp::browser_protocol::input::{
            DispatchMouseEventParams, DispatchMouseEventType, MouseButton as CdpMouseButton,
        };

        let cdp_button = match button {
            MouseButton::Left => CdpMouseButton::Left,
            MouseButton::Middle => CdpMouseButton::Middle,
            MouseButton::Right => CdpMouseButton::Right,
        };

        let params = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseReleased)
            .x(0.0)
            .y(0.0)
            .button(cdp_button)
            .click_count(1)
            .build()
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to build params: {}", e),
            })?;

        self.chaser
            .raw_page()
            .execute(params)
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Mouse up failed: {}", e),
            })?;
        Ok(())
    }

    /// Scroll using mouse wheel.
    pub async fn wheel(&self, delta_x: f64, delta_y: f64) -> AgentResult<()> {
        use crate::cdp::browser_protocol::input::{DispatchMouseEventParams, DispatchMouseEventType};

        let params = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseWheel)
            .x(0.0)
            .y(0.0)
            .delta_x(delta_x)
            .delta_y(delta_y)
            .build()
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to build params: {}", e),
            })?;

        self.chaser
            .raw_page()
            .execute(params)
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Wheel failed: {}", e),
            })?;
        Ok(())
    }

    /// Scroll the page or element.
    pub async fn scroll(&self, direction: ScrollDirection, amount: i32) -> AgentResult<()> {
        let (_delta_x, delta_y) = direction.to_deltas(amount);
        self.chaser.scroll_human(delta_y).await.map_err(|e| AgentError::Internal {
            message: format!("Scroll failed: {}", e),
        })?;
        Ok(())
    }

    /// Scroll an element into view.
    pub async fn scroll_into_view(&self, selector: &str) -> AgentResult<()> {
        let element = self.find_element(selector).await?;
        element
            .scroll_into_view()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Scroll into view failed: {}", e),
            })?;
        Ok(())
    }

    // =========================================================================
    // JavaScript Execution
    // =========================================================================

    /// Evaluate JavaScript and return the result.
    pub async fn evaluate(&self, expression: &str) -> AgentResult<serde_json::Value> {
        let result = self
            .chaser
            .evaluate(expression)
            .await
            .map_err(|e| AgentError::JavaScript {
                message: e.to_string(),
            })?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    // =========================================================================
    // Screenshots & PDFs
    // =========================================================================

    /// Take a screenshot.
    pub async fn screenshot(&self, options: ScreenshotOptions) -> AgentResult<ScreenshotData> {
        use crate::cdp::browser_protocol::page::{CaptureScreenshotFormat, CaptureScreenshotParams};

        let format = match options.format {
            ScreenshotFormat::Png => CaptureScreenshotFormat::Png,
            ScreenshotFormat::Jpeg => CaptureScreenshotFormat::Jpeg,
            ScreenshotFormat::Webp => CaptureScreenshotFormat::Webp,
        };

        let mut params = CaptureScreenshotParams::builder().format(format);

        if let Some(quality) = options.quality {
            params = params.quality(quality as i64);
        }

        let result = self
            .chaser
            .raw_page()
            .execute(params.build())
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Screenshot failed: {}", e),
            })?;

        if let Some(path) = options.path {
            // Save to file - result.data is already base64 encoded
            use base64::Engine;
            let data_bytes: &[u8] = result.data.as_ref();
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(data_bytes)
                .map_err(|e| AgentError::Internal {
                    message: format!("Failed to decode screenshot: {}", e),
                })?;
            std::fs::write(&path, decoded).map_err(|e| AgentError::Internal {
                message: format!("Failed to write screenshot: {}", e),
            })?;
            Ok(ScreenshotData::Path { path })
        } else {
            // Return as base64 string - data is already base64
            let data_bytes: &[u8] = result.data.as_ref();
            Ok(ScreenshotData::Base64 {
                data: String::from_utf8_lossy(data_bytes).to_string(),
            })
        }
    }

    // =========================================================================
    // Wait Methods
    // =========================================================================

    /// Wait for an element to reach a state.
    pub async fn wait_for(&self, selector: &str, options: WaitOptions) -> AgentResult<()> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(options.timeout);

        loop {
            let found = match options.state {
                ElementState::Visible => self.is_visible(&css).await.unwrap_or(false),
                ElementState::Hidden => !self.is_visible(&css).await.unwrap_or(true),
                ElementState::Attached => self.get_count(&css).await.unwrap_or(0) > 0,
                ElementState::Detached => self.get_count(&css).await.unwrap_or(1) == 0,
            };

            if found {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(AgentError::Timeout {
                    waiting_for: format!("{} to be {:?}", selector, options.state),
                    timeout_ms: options.timeout,
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Wait for navigation to complete.
    pub async fn wait_for_navigation(&self) -> AgentResult<()> {
        self.chaser
            .raw_page()
            .wait_for_navigation()
            .await
            .map_err(|e| AgentError::Navigation {
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Wait for a URL pattern.
    pub async fn wait_for_url(&self, pattern: &str, timeout_ms: u64) -> AgentResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            let url = self.get_url().await.unwrap_or_default();
            if url.contains(pattern) {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(AgentError::Timeout {
                    waiting_for: format!("URL to contain '{}'", pattern),
                    timeout_ms,
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Wait for a specific load state.
    pub async fn wait_for_load_state(&self, state: LoadState, timeout_ms: u64) -> AgentResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        let js = match state {
            LoadState::Load => "document.readyState === 'complete'",
            LoadState::DomContentLoaded => {
                "document.readyState === 'interactive' || document.readyState === 'complete'"
            }
            LoadState::NetworkIdle => {
                // Check for network idle using performance API
                r#"
                (function() {
                    const entries = performance.getEntriesByType('resource');
                    const recent = entries.filter(e => Date.now() - e.responseEnd < 500);
                    return recent.length === 0 && document.readyState === 'complete';
                })()
                "#
            }
        };

        loop {
            let result = self.chaser.evaluate(js).await.map_err(|e| AgentError::JavaScript {
                message: e.to_string(),
            })?;

            if result.and_then(|v| v.as_bool()).unwrap_or(false) {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(AgentError::Timeout {
                    waiting_for: format!("{:?} state", state),
                    timeout_ms,
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Wait for a JavaScript expression to return true.
    pub async fn wait_for_function(&self, expression: &str, timeout_ms: u64) -> AgentResult<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            let result = self
                .chaser
                .evaluate(expression)
                .await
                .map_err(|e| AgentError::JavaScript {
                    message: e.to_string(),
                })?;

            // Check if result is truthy
            let is_truthy = match result {
                Some(serde_json::Value::Bool(b)) => b,
                Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                Some(serde_json::Value::String(s)) => !s.is_empty(),
                Some(serde_json::Value::Array(a)) => !a.is_empty(),
                Some(serde_json::Value::Object(_)) => true,
                Some(serde_json::Value::Null) | None => false,
            };

            if is_truthy {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(AgentError::Timeout {
                    waiting_for: format!("expression '{}' to be truthy", expression),
                    timeout_ms,
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    // =========================================================================
    // Internal Helpers
    // =========================================================================

    /// Find an element by selector or ref.
    async fn find_element(&self, selector: &str) -> AgentResult<crate::Element> {
        let locator = Locator::parse(selector);
        let css = self.locator_to_selector(&locator).await?;

        self.chaser
            .raw_page()
            .find_element(&css)
            .await
            .map_err(|_| AgentError::ElementNotFound {
                selector: selector.to_string(),
            })
    }

    /// Convert a locator to a CSS selector string.
    async fn locator_to_selector(&self, locator: &Locator) -> AgentResult<String> {
        let ref_map = self.ref_map.lock().await;
        locator.to_css_selector(Some(&ref_map))
    }
}

impl std::fmt::Debug for AgentPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentPage")
            .field("track_console", &self.track_console)
            .field("track_errors", &self.track_errors)
            .finish()
    }
}
