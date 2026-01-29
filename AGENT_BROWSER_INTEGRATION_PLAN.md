# Agent-Browser Integration Plan for chaser-oxide

> **Goal:** 100% feature parity with [agent-browser](https://github.com/vercel-labs/agent-browser) - a headless browser automation CLI for AI agents with 108+ commands across 16 categories.

## Overview

This plan integrates agent-browser's AI-first design philosophy into chaser-oxide's Rust-based stealth browser automation library. The key innovation is the **Snapshot + Refs** system that reduces AI context usage by up to 93%.

---

## Phase 0: Foundation & Infrastructure
*Core protocol and architectural changes*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 0.1 | Create `agent_browser` module structure (`src/agent_browser/mod.rs`) | - | P0 |
| 0.2 | Define `AgentCommand` enum (discriminated union of all 108+ commands) | 0.1 | P0 |
| 0.3 | Define `AgentResponse<T>` result type with success/error discriminant | 0.2 | P0 |
| 0.4 | Create `AgentBrowser` main struct wrapping `Browser` | 0.1 | P0 |
| 0.5 | Create `AgentPage` struct wrapping `ChaserPage` with ref system | 0.4 | P0 |
| 0.6 | Implement JSON command parser with Zod-like validation (use `serde`) | 0.2, 0.3 | P0 |
| 0.7 | Implement JSON response serializer | 0.3 | P0 |
| 0.8 | Add `--json` output mode flag for machine-readable responses | 0.7 | P1 |
| 0.9 | Create unified error types with agent-browser error codes | 0.3 | P0 |
| 0.10 | Add feature flag `agent-browser` for conditional compilation | 0.1 | P0 |

---

## Phase 1: Snapshot & Refs System ⭐
*Core innovation - accessibility tree with element references*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 1.1 | Create `Snapshot` struct with accessibility tree representation | 0.5 | P0 |
| 1.2 | Create `RefMap` type (`HashMap<String, RefInfo>`) for ref→selector mapping | 1.1 | P0 |
| 1.3 | Define `RefInfo` struct: `{ selector, role, name, nth }` | 1.2 | P0 |
| 1.4 | Implement `snapshot()` command using CDP `Accessibility.getFullAXTree` | 1.1 | P0 |
| 1.5 | Implement ref ID generation (`e1`, `e2`, etc.) with sequential counter | 1.4 | P0 |
| 1.6 | Classify element roles: interactive, content, structural | 1.4 | P0 |
| 1.7 | Implement `-i` (interactive only) filter | 1.6 | P0 |
| 1.8 | Implement `-c` (compact) filter - remove unnamed structural elements | 1.6 | P0 |
| 1.9 | Implement `-d N` (max depth) filter | 1.4 | P0 |
| 1.10 | Implement `-s selector` (CSS selector scope) filter | 1.4 | P0 |
| 1.11 | Implement ref caching (store last snapshot's RefMap) | 1.2 | P0 |
| 1.12 | Implement `@ref` notation parser (detect `@e1`, `@e2`, etc.) | 1.11 | P0 |
| 1.13 | Implement `get_locator_from_ref()` - resolve ref to Playwright-style locator | 1.12 | P0 |
| 1.14 | Add deduplication handling (nth index for identical role+name) | 1.5 | P1 |
| 1.15 | Generate hierarchical tree output with indentation | 1.4 | P1 |
| 1.16 | Add metadata annotations (`[level=1]` for headings, etc.) | 1.15 | P2 |

---

## Phase 2: Semantic Locators
*Role-based, text-based, and accessibility-aware element finding*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 2.1 | Implement `get_by_role(role, name?, exact?)` | 0.5 | P0 |
| 2.2 | Implement `get_by_text(text, exact?)` | 0.5 | P0 |
| 2.3 | Implement `get_by_label(label, exact?)` | 0.5 | P0 |
| 2.4 | Implement `get_by_placeholder(text, exact?)` | 0.5 | P0 |
| 2.5 | Implement `get_by_alt_text(text, exact?)` | 0.5 | P0 |
| 2.6 | Implement `get_by_title(title, exact?)` | 0.5 | P0 |
| 2.7 | Implement `get_by_test_id(testId)` - data-testid attribute | 0.5 | P0 |
| 2.8 | Implement `nth(index)` for indexed element selection (-1 = last) | 2.1-2.7 | P0 |
| 2.9 | Create unified `Locator` type that works with refs AND semantic locators | 2.1-2.8, 1.13 | P0 |
| 2.10 | Implement locator chaining (e.g., `get_by_role().get_by_text()`) | 2.9 | P1 |
| 2.11 | Add combined semantic actions: `click_by_role(role, name)`, etc. | 2.1-2.7 | P1 |

---

## Phase 3: Element Actions
*All interaction commands*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 3.1 | Implement `click(selector, button?, clickCount?, delay?)` | 2.9 | P0 |
| 3.2 | Implement `dblclick(selector)` | 3.1 | P0 |
| 3.3 | Implement `hover(selector)` | 2.9 | P0 |
| 3.4 | Implement `tap(selector)` - touch interaction | 2.9 | P0 |
| 3.5 | Implement `focus(selector)` | 2.9 | P0 |
| 3.6 | Implement `type(selector, text, clear?)` - sequential typing | 2.9 | P0 |
| 3.7 | Implement `fill(selector, value)` - direct value set | 2.9 | P0 |
| 3.8 | Implement `clear(selector)` - clear input | 2.9 | P0 |
| 3.9 | Implement `select_all(selector)` - select all text | 2.9 | P0 |
| 3.10 | Implement `check(selector)` - checkbox/radio | 2.9 | P0 |
| 3.11 | Implement `uncheck(selector)` | 3.10 | P0 |
| 3.12 | Implement `select(selector, value/values)` - dropdown | 2.9 | P0 |
| 3.13 | Implement `multi_select(selector, values[])` | 3.12 | P1 |
| 3.14 | Implement `upload(selector, files[])` - file input | 2.9 | P0 |
| 3.15 | Implement `drag(source, target)` - drag and drop | 2.9 | P1 |
| 3.16 | Implement `dispatch_event(selector, event, eventInit?)` | 2.9 | P1 |
| 3.17 | Implement `highlight(selector)` - visual debugging | 2.9 | P2 |
| 3.18 | Implement `set_value(selector, value)` - alias for fill | 3.7 | P2 |

---

## Phase 4: Keyboard & Mouse Control
*Low-level input simulation*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 4.1 | Implement `press(key, selector?)` - single key press | 0.5 | P0 |
| 4.2 | Implement `keyboard(shortcut)` - key combinations (Control+a, etc.) | 4.1 | P0 |
| 4.3 | Implement `key_down(key)` - hold key | 4.1 | P0 |
| 4.4 | Implement `key_up(key)` - release key | 4.1 | P0 |
| 4.5 | Implement `insert_text(text)` - insert without key events | 0.5 | P1 |
| 4.6 | Implement `mouse_move(x, y)` | 0.5 | P0 |
| 4.7 | Implement `mouse_down(button?, clickCount?)` | 4.6 | P0 |
| 4.8 | Implement `mouse_up(button?)` | 4.6 | P0 |
| 4.9 | Implement `wheel(deltaX?, deltaY?)` - mouse wheel | 4.6 | P0 |
| 4.10 | Implement `scroll(direction_or_coords, selector?)` | 0.5 | P0 |
| 4.11 | Implement `scroll_into_view(selector)` | 2.9 | P0 |

---

## Phase 5: Information Retrieval
*Query and inspection commands*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 5.1 | Implement `get_text(selector)` - visible text | 2.9 | P0 |
| 5.2 | Implement `get_html(selector?)` - inner HTML | 2.9 | P0 |
| 5.3 | Implement `get_value(selector)` - input value | 2.9 | P0 |
| 5.4 | Implement `get_attribute(selector, attr)` | 2.9 | P0 |
| 5.5 | Implement `get_title()` - page title | 0.5 | P0 |
| 5.6 | Implement `get_url()` - current URL | 0.5 | P0 |
| 5.7 | Implement `get_count(selector)` - element count | 2.9 | P0 |
| 5.8 | Implement `get_bounding_box(selector)` - element rectangle | 2.9 | P0 |
| 5.9 | Implement `get_styles(selector)` - computed styles + box | 2.9 | P1 |
| 5.10 | Implement `inner_text(selector)` | 5.1 | P1 |
| 5.11 | Implement `inner_html(selector)` | 5.2 | P1 |
| 5.12 | Implement `content(selector?)` - full HTML | 0.5 | P0 |

---

## Phase 6: State Checking
*Element state inspection*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 6.1 | Implement `is_visible(selector)` | 2.9 | P0 |
| 6.2 | Implement `is_enabled(selector)` | 2.9 | P0 |
| 6.3 | Implement `is_checked(selector)` | 2.9 | P0 |
| 6.4 | Implement `is_editable(selector)` | 2.9 | P1 |
| 6.5 | Implement `is_hidden(selector)` | 6.1 | P1 |

---

## Phase 7: Wait Mechanisms
*Conditional waiting*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 7.1 | Implement `wait(selector, state?, timeout?)` - wait for selector | 2.9 | P0 |
| 7.2 | Implement `wait_for_url(url_pattern, timeout?)` | 0.5 | P0 |
| 7.3 | Implement `wait_for_load_state(state)` - load/domcontentloaded/networkidle | 0.5 | P0 |
| 7.4 | Implement `wait_for_function(expression, timeout?)` - JS condition | 0.5 | P0 |
| 7.5 | Implement `wait_for_download(timeout?)` | 0.5 | P1 |
| 7.6 | Implement timeout with millisecond precision | 7.1-7.5 | P0 |

---

## Phase 8: Navigation
*Page navigation commands*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 8.1 | Implement `open(url, waitUntil?, headers?)` / `navigate` / `goto` | 0.5 | P0 |
| 8.2 | Implement `back()` - history back | 0.5 | P0 |
| 8.3 | Implement `forward()` - history forward | 0.5 | P0 |
| 8.4 | Implement `reload()` | 0.5 | P0 |
| 8.5 | Implement `close()` - close page | 0.5 | P0 |

---

## Phase 9: Browser Settings & Emulation
*Device, viewport, and environment configuration*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 9.1 | Implement `set_viewport(width, height)` | 0.5 | P0 |
| 9.2 | Implement `set_device(deviceName)` - predefined device profiles | 9.1 | P0 |
| 9.3 | Implement `list_devices()` - list available device names | 9.2 | P1 |
| 9.4 | Implement `set_geolocation(lat, long, accuracy?)` | 0.5 | P0 |
| 9.5 | Implement `set_offline(offline)` | 0.5 | P0 |
| 9.6 | Implement `set_headers(headers, origin?)` | 0.5 | P0 |
| 9.7 | Implement `set_credentials(username, password, origin?)` - HTTP auth | 0.5 | P0 |
| 9.8 | Implement `set_permissions(permissions[], grant)` | 0.5 | P1 |
| 9.9 | Implement `emulate_media(type?, colorScheme?, reducedMotion?, forcedColors?)` | 0.5 | P1 |
| 9.10 | Implement `set_user_agent(ua)` (launch-time only warning) | 0.4 | P1 |
| 9.11 | Implement `set_timezone(tz)` (launch-time only warning) | 0.4 | P1 |
| 9.12 | Implement `set_locale(locale)` (launch-time only warning) | 0.4 | P1 |
| 9.13 | Implement scoped headers (per-origin) | 9.6 | P1 |

---

## Phase 10: Cookies & Storage
*Persistent state management*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 10.1 | Implement `cookies_get(urls?)` | 0.5 | P0 |
| 10.2 | Implement `cookies_set(cookies[])` - auto-fills URL if missing | 0.5 | P0 |
| 10.3 | Implement `cookies_clear()` | 0.5 | P0 |
| 10.4 | Implement `storage_get(key?, type?)` - localStorage/sessionStorage | 0.5 | P0 |
| 10.5 | Implement `storage_set(key, value, type?)` | 0.5 | P0 |
| 10.6 | Implement `storage_clear(type?)` | 0.5 | P0 |
| 10.7 | Implement `state_save(path)` - export storage state | 0.5 | P1 |
| 10.8 | Implement `state_load(path)` - import storage state (launch-time) | 0.4 | P1 |

---

## Phase 11: Network
*Request interception and monitoring*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 11.1 | Implement `route(urlPattern, response_or_abort)` - mock/intercept | 0.5 | P0 |
| 11.2 | Implement `unroute(urlPattern?)` - remove route | 11.1 | P0 |
| 11.3 | Implement `requests(filter?)` - get tracked requests | 0.5 | P0 |
| 11.4 | Implement request tracking start/stop | 11.3 | P0 |
| 11.5 | Implement `response_body(urlPattern)` - wait and extract response | 0.5 | P1 |
| 11.6 | Implement `download(urlPattern, savePath)` - file download | 0.5 | P1 |
| 11.7 | Implement `abort(requestId)` - abort specific request | 11.1 | P1 |
| 11.8 | Implement `fulfill(requestId, body, status?, headers?)` | 11.1 | P0 |

---

## Phase 12: Tabs & Windows
*Multi-tab/window management*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 12.1 | Implement `tab_new(url?)` - create new tab | 0.4 | P0 |
| 12.2 | Implement `tab_list()` - list tabs with index/url/title/active | 0.4 | P0 |
| 12.3 | Implement `tab_switch(index)` - switch to tab by index | 12.2 | P0 |
| 12.4 | Implement `tab_close(index?)` - close tab | 12.2 | P0 |
| 12.5 | Implement `window_new(viewport?)` - new window | 0.4 | P1 |
| 12.6 | Implement `bring_to_front()` - focus window | 0.5 | P1 |

---

## Phase 13: Frames
*iframe navigation*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 13.1 | Implement `frame(selector_or_name_or_url)` - switch to iframe | 0.5 | P0 |
| 13.2 | Implement `main_frame()` - return to main frame | 13.1 | P0 |
| 13.3 | Track current frame context | 13.1 | P0 |

---

## Phase 14: Dialogs
*Alert/confirm/prompt handling*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 14.1 | Implement `dialog(accept_or_dismiss, promptText?)` - set handler | 0.5 | P0 |
| 14.2 | Implement dialog auto-handling with configurable defaults | 14.1 | P1 |
| 14.3 | Clear dialog handler | 14.1 | P1 |

---

## Phase 15: JavaScript Execution
*Script evaluation and injection*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 15.1 | Implement `evaluate(expression)` - run JS, return result | 0.5 | P0 |
| 15.2 | Implement `evaluate_handle(expression)` - return object handle | 15.1 | P1 |
| 15.3 | Implement `add_init_script(script)` - run on every navigation | 0.5 | P0 |
| 15.4 | Implement `add_script(content_or_url)` - inject script tag | 0.5 | P1 |
| 15.5 | Implement `add_style(content_or_url)` - inject style tag | 0.5 | P1 |
| 15.6 | Implement `expose_function(name, fn)` - expose to page context | 0.5 | P2 |
| 15.7 | Implement `set_content(html)` - replace page HTML | 0.5 | P1 |

---

## Phase 16: Screenshots & PDFs
*Visual capture*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 16.1 | Implement `screenshot(path?, selector?, fullPage?, format?, quality?)` | 0.5 | P0 |
| 16.2 | Implement `pdf(path?, format?)` | 0.5 | P0 |
| 16.3 | Support base64 return for screenshot | 16.1 | P1 |

---

## Phase 17: Recording & Tracing
*Debugging and replay*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 17.1 | Implement `trace_start(screenshots?, snapshots?)` | 0.5 | P1 |
| 17.2 | Implement `trace_stop(path)` | 17.1 | P1 |
| 17.3 | Implement `har_start()` - begin HAR recording | 0.5 | P1 |
| 17.4 | Implement `har_stop(path)` - save HAR file | 17.3 | P1 |
| 17.5 | Implement `video_start()` (launch-time only) | 0.4 | P2 |
| 17.6 | Implement `video_stop()` - get video path | 17.5 | P2 |
| 17.7 | Implement `recording_start(path, url?)` - native video recording | 0.5 | P2 |
| 17.8 | Implement `recording_stop()` | 17.7 | P2 |
| 17.9 | Implement `recording_restart(path)` | 17.7 | P2 |

---

## Phase 18: Console & Error Tracking
*Logging and diagnostics*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 18.1 | Implement `console(clear?)` - get/clear console messages | 0.5 | P0 |
| 18.2 | Implement `errors(clear?)` - get/clear page errors | 0.5 | P0 |
| 18.3 | Start console tracking on page creation | 18.1 | P0 |
| 18.4 | Start error tracking on page creation | 18.2 | P0 |
| 18.5 | Implement `pause()` - debugger pause | 0.5 | P2 |

---

## Phase 19: Clipboard
*Copy/paste operations*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 19.1 | Implement `clipboard(action)` - copy/paste/read | 0.5 | P1 |

---

## Phase 20: Streaming Server ⭐
*WebSocket viewport streaming and input injection*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 20.1 | Create `StreamServer` struct with WebSocket server | 0.5 | P1 |
| 20.2 | Implement screencast start using CDP `Page.startScreencast` | 20.1 | P1 |
| 20.3 | Implement frame broadcasting (base64 JPEG) to connected clients | 20.2 | P1 |
| 20.4 | Add frame metadata: scroll position, viewport dimensions, scale | 20.3 | P1 |
| 20.5 | Implement `screencast_start(format?, quality?, maxWidth?, maxHeight?)` | 20.2 | P1 |
| 20.6 | Implement `screencast_stop()` | 20.5 | P1 |
| 20.7 | Implement mouse input injection from WebSocket clients | 20.1 | P1 |
| 20.8 | Implement keyboard input injection from WebSocket clients | 20.1 | P1 |
| 20.9 | Implement touch input injection from WebSocket clients | 20.1 | P2 |
| 20.10 | Add origin validation for WebSocket security | 20.1 | P1 |
| 20.11 | Configurable stream port via environment variable | 20.1 | P1 |

---

## Phase 21: Raw Input Injection (CDP)
*Low-level input via Chrome DevTools Protocol*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 21.1 | Implement `input_mouse(type, x, y, button?, clickCount?, deltaX?, deltaY?, modifiers?)` | 0.5 | P1 |
| 21.2 | Implement `input_keyboard(type, key?, code?, text?, modifiers?)` | 0.5 | P1 |
| 21.3 | Implement `input_touch(type, touchPoints[], modifiers?)` | 0.5 | P2 |
| 21.4 | Get or create CDP session for raw input | 21.1-21.3 | P1 |

---

## Phase 22: Session & Profile Management
*Isolation and persistence*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 22.1 | Implement `--session` flag for isolated browser instances | 0.4 | P0 |
| 22.2 | Implement session-based socket/port separation | 22.1 | P0 |
| 22.3 | Implement `--profile` flag for persistent user data dir | 0.4 | P0 |
| 22.4 | Auto-save cookies/storage on session end | 22.3 | P1 |
| 22.5 | Implement PID file management for daemon detection | 22.1 | P1 |

---

## Phase 23: Launch & Connection
*Browser lifecycle management*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 23.1 | Implement `launch(options)` with all agent-browser options | 0.4 | P0 |
| 23.2 | Support headless mode configuration | 23.1 | P0 |
| 23.3 | Support custom executable path | 23.1 | P0 |
| 23.4 | Support proxy configuration | 23.1 | P0 |
| 23.5 | Support extensions loading (Chromium) | 23.1 | P1 |
| 23.6 | Support additional browser args | 23.1 | P0 |
| 23.7 | Implement `connect_via_cdp(endpoint)` - connect to existing browser | 0.4 | P0 |
| 23.8 | Support `--cdp` flag (port number or WebSocket URL) | 23.7 | P0 |
| 23.9 | Auto-launch browser if not running (daemon behavior) | 23.1 | P1 |
| 23.10 | Graceful shutdown on SIGINT/SIGTERM/SIGHUP | 0.4 | P1 |

---

## Phase 24: Cloud Provider Integration
*Remote browser infrastructure*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 24.1 | Create `CloudProvider` trait | 23.7 | P2 |
| 24.2 | Implement Browserbase provider | 24.1 | P2 |
| 24.3 | Implement Kernel provider (stealth mode, persistent profiles) | 24.1 | P2 |
| 24.4 | Implement Browser Use provider | 24.1 | P2 |
| 24.5 | Support `-p` flag for provider selection | 24.1 | P2 |
| 24.6 | Support `AGENT_BROWSER_PROVIDER` environment variable | 24.5 | P2 |
| 24.7 | Support provider-specific environment variables | 24.2-24.4 | P2 |

---

## Phase 25: CLI Interface (Optional)
*Command-line tool for agent consumption*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 25.1 | Create `chaser-agent` binary crate | All P0 tasks | P2 |
| 25.2 | Implement command parsing with clap | 25.1 | P2 |
| 25.3 | Implement JSON output mode | 25.1 | P2 |
| 25.4 | Implement line-delimited JSON protocol | 25.1 | P2 |
| 25.5 | Support stdin command reading | 25.4 | P2 |
| 25.6 | Create daemon mode with socket communication | 25.4 | P2 |

---

## Phase 26: Testing & Documentation
*Quality assurance*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 26.1 | Unit tests for RefMap and snapshot generation | Phase 1 | P0 |
| 26.2 | Unit tests for semantic locators | Phase 2 | P0 |
| 26.3 | Integration tests for all 108+ commands | All | P1 |
| 26.4 | Test ref persistence across snapshots | 1.11 | P1 |
| 26.5 | Test session isolation | 22.1 | P1 |
| 26.6 | Document API in rustdoc | All | P1 |
| 26.7 | Create migration guide from chromiumoxide | All | P2 |
| 26.8 | Create examples for common AI agent patterns | All | P1 |

---

## Phase 27: Performance Optimization
*AI-focused optimizations*

| ID | Task | Dependencies | Priority |
|----|------|--------------|----------|
| 27.1 | Benchmark snapshot generation time | Phase 1 | P2 |
| 27.2 | Optimize RefMap memory usage | 1.2 | P2 |
| 27.3 | Implement lazy snapshot filtering | 1.7-1.10 | P2 |
| 27.4 | Add snapshot caching with invalidation | 1.11 | P2 |
| 27.5 | Optimize JSON serialization for large responses | 0.7 | P2 |

---

## Dependency Graph (Critical Path)

```
Phase 0 (Foundation)
    ↓
Phase 1 (Snapshot/Refs) ←──── Core Innovation
    ↓
Phase 2 (Semantic Locators)
    ↓
Phase 3-6 (Element Actions + Info + State)
    ↓
Phase 7-8 (Wait + Navigation)
    ↓
Phase 9-14 (Browser Settings + Storage + Network + Tabs + Frames + Dialogs)
    ↓
Phase 15-19 (JS + Screenshots + Recording + Console + Clipboard)
    ↓
Phase 20-21 (Streaming + Raw Input)
    ↓
Phase 22-24 (Session + Launch + Cloud)
    ↓
Phase 25-27 (CLI + Testing + Optimization)
```

---

## Priority Legend

| Priority | Meaning | Count |
|----------|---------|-------|
| **P0** | Critical - Core functionality | ~45 tasks |
| **P1** | Important - Full feature parity | ~50 tasks |
| **P2** | Nice-to-have - Advanced features | ~35 tasks |

---

## Estimated Task Count by Phase

| Phase | Tasks | P0 | P1 | P2 |
|-------|-------|----|----|----|
| 0. Foundation | 10 | 8 | 2 | 0 |
| 1. Snapshot/Refs | 16 | 13 | 2 | 1 |
| 2. Semantic Locators | 11 | 9 | 2 | 0 |
| 3. Element Actions | 18 | 14 | 2 | 2 |
| 4. Keyboard/Mouse | 11 | 9 | 2 | 0 |
| 5. Info Retrieval | 12 | 10 | 2 | 0 |
| 6. State Checking | 5 | 3 | 2 | 0 |
| 7. Wait Mechanisms | 6 | 5 | 1 | 0 |
| 8. Navigation | 5 | 5 | 0 | 0 |
| 9. Browser Settings | 13 | 7 | 6 | 0 |
| 10. Cookies/Storage | 8 | 6 | 2 | 0 |
| 11. Network | 8 | 4 | 4 | 0 |
| 12. Tabs/Windows | 6 | 4 | 2 | 0 |
| 13. Frames | 3 | 3 | 0 | 0 |
| 14. Dialogs | 3 | 1 | 2 | 0 |
| 15. JavaScript | 7 | 2 | 4 | 1 |
| 16. Screenshots/PDFs | 3 | 2 | 1 | 0 |
| 17. Recording/Tracing | 9 | 0 | 4 | 5 |
| 18. Console/Errors | 5 | 4 | 0 | 1 |
| 19. Clipboard | 1 | 0 | 1 | 0 |
| 20. Streaming Server | 11 | 0 | 9 | 2 |
| 21. Raw Input | 4 | 0 | 3 | 1 |
| 22. Session/Profile | 5 | 2 | 3 | 0 |
| 23. Launch/Connection | 10 | 5 | 5 | 0 |
| 24. Cloud Providers | 7 | 0 | 0 | 7 |
| 25. CLI Interface | 6 | 0 | 0 | 6 |
| 26. Testing/Docs | 8 | 2 | 5 | 1 |
| 27. Performance | 5 | 0 | 0 | 5 |
| **TOTAL** | **~200** | **~113** | **~66** | **~31** |

---

## Sources

- [agent-browser GitHub](https://github.com/vercel-labs/agent-browser)
- [agent-browser.dev](https://agent-browser.dev/)
