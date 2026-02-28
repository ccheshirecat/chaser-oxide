//! AgentBrowser - Browser-level agent interface for tab/window management.
//!
//! Wraps `Browser` to provide multi-tab, session, and lifecycle management
//! compatible with the agent-browser protocol.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::browser::Browser;
use crate::chaser::ChaserPage;

use super::agent_page::AgentPage;
use super::commands::ViewportSize;
use super::response::*;

/// AgentBrowser provides browser-level management for AI agents.
///
/// It wraps `Browser` and manages tabs, windows, sessions, and browser lifecycle.
pub struct AgentBrowser {
    /// The underlying Browser instance.
    browser: Browser,

    /// List of pages managed as tabs, in order.
    pages: Arc<Mutex<Vec<AgentPage>>>,

    /// Index of the currently active tab.
    active_tab: Arc<Mutex<usize>>,
}

impl AgentBrowser {
    /// Create a new AgentBrowser wrapping a Browser.
    pub fn new(browser: Browser) -> Self {
        Self {
            browser,
            pages: Arc::new(Mutex::new(Vec::new())),
            active_tab: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the underlying Browser reference.
    pub fn browser(&self) -> &Browser {
        &self.browser
    }

    /// Get mutable access to the underlying Browser.
    pub fn browser_mut(&mut self) -> &mut Browser {
        &mut self.browser
    }

    // =========================================================================
    // Tabs & Windows (Phase 12)
    // =========================================================================

    /// Create a new tab, optionally navigating to a URL.
    pub async fn tab_new(&self, url: Option<&str>) -> AgentResult<AgentPage> {
        let target_url = url.unwrap_or("about:blank");
        let page = self
            .browser
            .new_page(target_url)
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to create new tab: {}", e),
            })?;

        let chaser = ChaserPage::new(page);
        let agent_page = AgentPage::new(chaser);

        // Add to tab list
        {
            let mut pages = self.pages.lock().await;
            pages.push(AgentPage::new(ChaserPage::new(
                self.browser
                    .get_page(agent_page.chaser().raw_page().target_id().clone())
                    .await
                    .map_err(|e| AgentError::Internal {
                        message: format!("Failed to get page handle: {}", e),
                    })?,
            )));
            let new_idx = pages.len() - 1;
            let mut active = self.active_tab.lock().await;
            *active = new_idx;
        }

        Ok(agent_page)
    }

    /// List all open tabs with their index, URL, title, and active status.
    pub async fn tab_list(&self) -> AgentResult<Vec<TabInfo>> {
        let browser_pages = self
            .browser
            .pages()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to list tabs: {}", e),
            })?;

        let active_idx = *self.active_tab.lock().await;

        let mut tabs = Vec::new();
        for (index, page) in browser_pages.iter().enumerate() {
            let url = page
                .url()
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "about:blank".to_string());
            let title = page.get_title().await.ok().flatten().unwrap_or_default();
            tabs.push(TabInfo {
                index,
                url,
                title,
                active: index == active_idx,
            });
        }

        Ok(tabs)
    }

    /// Switch to a tab by index.
    pub async fn tab_switch(&self, index: usize) -> AgentResult<AgentPage> {
        let browser_pages = self
            .browser
            .pages()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to list tabs: {}", e),
            })?;

        if index >= browser_pages.len() {
            return Err(AgentError::InvalidCommand {
                message: format!(
                    "Tab index {} out of range (0-{})",
                    index,
                    browser_pages.len().saturating_sub(1)
                ),
            });
        }

        let page = &browser_pages[index];
        page.bring_to_front()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to switch to tab: {}", e),
            })?;

        // Update active tab index
        {
            let mut active = self.active_tab.lock().await;
            *active = index;
        }

        // Return a new AgentPage for this tab
        let cloned_page = self
            .browser
            .get_page(page.target_id().clone())
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to get page: {}", e),
            })?;
        Ok(AgentPage::new(ChaserPage::new(cloned_page)))
    }

    /// Close a tab by index (defaults to active tab).
    pub async fn tab_close(&self, index: Option<usize>) -> AgentResult<()> {
        let browser_pages = self
            .browser
            .pages()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to list tabs: {}", e),
            })?;

        let target_index = index.unwrap_or(*self.active_tab.lock().await);

        if target_index >= browser_pages.len() {
            return Err(AgentError::InvalidCommand {
                message: format!(
                    "Tab index {} out of range (0-{})",
                    target_index,
                    browser_pages.len().saturating_sub(1)
                ),
            });
        }

        let page = browser_pages.into_iter().nth(target_index).unwrap();
        page.close().await.map_err(|e| AgentError::Internal {
            message: format!("Failed to close tab: {}", e),
        })?;

        // Adjust active tab index
        {
            let mut active = self.active_tab.lock().await;
            if *active >= target_index && *active > 0 {
                *active -= 1;
            }
        }

        Ok(())
    }

    /// Create a new browser window with optional viewport.
    pub async fn window_new(&self, viewport: Option<ViewportSize>) -> AgentResult<AgentPage> {
        let page =
            self.browser
                .new_page("about:blank")
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Failed to create new window: {}", e),
                })?;

        // If viewport is specified, set device metrics
        if let Some(vp) = viewport {
            use crate::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;

            let params = SetDeviceMetricsOverrideParams::builder()
                .width(vp.width as i64)
                .height(vp.height as i64)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .map_err(|e| AgentError::Internal {
                    message: format!("Failed to build viewport params: {}", e),
                })?;

            page.execute(params)
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Failed to set viewport: {}", e),
                })?;
        }

        let chaser = ChaserPage::new(page);
        Ok(AgentPage::new(chaser))
    }

    /// Bring the current page/window to front.
    pub async fn bring_to_front(&self) -> AgentResult<()> {
        let browser_pages = self
            .browser
            .pages()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to list pages: {}", e),
            })?;

        let active_idx = *self.active_tab.lock().await;
        if let Some(page) = browser_pages.get(active_idx) {
            page.bring_to_front()
                .await
                .map_err(|e| AgentError::Internal {
                    message: format!("Failed to bring to front: {}", e),
                })?;
        }
        Ok(())
    }

    /// Get the currently active AgentPage.
    pub async fn active_page(&self) -> AgentResult<AgentPage> {
        let browser_pages = self
            .browser
            .pages()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to list pages: {}", e),
            })?;

        let active_idx = *self.active_tab.lock().await;
        let page =
            browser_pages
                .into_iter()
                .nth(active_idx)
                .ok_or_else(|| AgentError::Internal {
                    message: "No active page".to_string(),
                })?;

        Ok(AgentPage::new(ChaserPage::new(page)))
    }

    // =========================================================================
    // Session Management (Phase 22)
    // =========================================================================

    /// Get the number of open tabs.
    pub async fn tab_count(&self) -> AgentResult<usize> {
        let pages = self
            .browser
            .pages()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to count tabs: {}", e),
            })?;
        Ok(pages.len())
    }

    /// Close the browser completely.
    pub async fn close(mut self) -> AgentResult<()> {
        self.browser
            .close()
            .await
            .map_err(|e| AgentError::Internal {
                message: format!("Failed to close browser: {}", e),
            })?;
        Ok(())
    }
}

impl std::fmt::Debug for AgentBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentBrowser").finish()
    }
}
