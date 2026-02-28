//! Raw Input Injection via CDP (Phase 21).
//!
//! Provides low-level input injection using the Chrome DevTools Protocol
//! Input domain for mouse, keyboard, and touch events.

use super::commands::{MouseButton, RawKeyboardInput, RawMouseInput, RawTouchInput};
use super::response::{AgentError, AgentResult};

use crate::page::Page;

/// Dispatch a raw mouse input event via CDP.
pub async fn dispatch_mouse_event(page: &Page, input: &RawMouseInput) -> AgentResult<()> {
    use crate::cdp::browser_protocol::input::{
        DispatchMouseEventParams, DispatchMouseEventType, MouseButton as CdpMouseButton,
    };

    let event_type = match input.event_type.as_str() {
        "mousePressed" => DispatchMouseEventType::MousePressed,
        "mouseReleased" => DispatchMouseEventType::MouseReleased,
        "mouseMoved" => DispatchMouseEventType::MouseMoved,
        "mouseWheel" => DispatchMouseEventType::MouseWheel,
        other => {
            return Err(AgentError::InvalidCommand {
                message: format!("Unknown mouse event type: {}", other),
            })
        }
    };

    let cdp_button = match input.button {
        MouseButton::Left => CdpMouseButton::Left,
        MouseButton::Middle => CdpMouseButton::Middle,
        MouseButton::Right => CdpMouseButton::Right,
    };

    let mut params = DispatchMouseEventParams::builder()
        .r#type(event_type)
        .x(input.x)
        .y(input.y)
        .button(cdp_button)
        .click_count(input.click_count as i64);

    if input.delta_x != 0.0 || input.delta_y != 0.0 {
        params = params.delta_x(input.delta_x).delta_y(input.delta_y);
    }

    let modifier_flags = input.modifiers.to_cdp_flags();
    if modifier_flags != 0 {
        params = params.modifiers(modifier_flags);
    }

    let built = params.build().map_err(|e| AgentError::Internal {
        message: format!("Failed to build mouse params: {}", e),
    })?;

    page.execute(built)
        .await
        .map_err(|e| AgentError::Internal {
            message: format!("Failed to dispatch mouse event: {}", e),
        })?;

    Ok(())
}

/// Dispatch a raw keyboard input event via CDP.
pub async fn dispatch_keyboard_event(page: &Page, input: &RawKeyboardInput) -> AgentResult<()> {
    use crate::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};

    let event_type = match input.event_type.as_str() {
        "keyDown" => DispatchKeyEventType::KeyDown,
        "keyUp" => DispatchKeyEventType::KeyUp,
        "char" => DispatchKeyEventType::Char,
        "rawKeyDown" => DispatchKeyEventType::RawKeyDown,
        other => {
            return Err(AgentError::InvalidCommand {
                message: format!("Unknown keyboard event type: {}", other),
            })
        }
    };

    let mut params = DispatchKeyEventParams::builder().r#type(event_type);

    if let Some(ref key) = input.key {
        params = params.key(key);
    }
    if let Some(ref code) = input.code {
        params = params.code(code);
    }
    if let Some(ref text) = input.text {
        params = params.text(text);
    }

    let modifier_flags = input.modifiers.to_cdp_flags();
    if modifier_flags != 0 {
        params = params.modifiers(modifier_flags);
    }

    let built = params.build().map_err(|e| AgentError::Internal {
        message: format!("Failed to build keyboard params: {}", e),
    })?;

    page.execute(built)
        .await
        .map_err(|e| AgentError::Internal {
            message: format!("Failed to dispatch keyboard event: {}", e),
        })?;

    Ok(())
}

/// Dispatch a raw touch input event via CDP.
pub async fn dispatch_touch_event(page: &Page, input: &RawTouchInput) -> AgentResult<()> {
    use crate::cdp::browser_protocol::input::{
        DispatchTouchEventParams, DispatchTouchEventType, TouchPoint as CdpTouchPoint,
    };

    let event_type = match input.event_type.as_str() {
        "touchStart" => DispatchTouchEventType::TouchStart,
        "touchMove" => DispatchTouchEventType::TouchMove,
        "touchEnd" => DispatchTouchEventType::TouchEnd,
        "touchCancel" => DispatchTouchEventType::TouchCancel,
        other => {
            return Err(AgentError::InvalidCommand {
                message: format!("Unknown touch event type: {}", other),
            })
        }
    };

    let touch_points: Vec<CdpTouchPoint> = input
        .touch_points
        .iter()
        .map(|tp| CdpTouchPoint::new(tp.x, tp.y))
        .collect();

    let mut params = DispatchTouchEventParams::builder()
        .r#type(event_type)
        .touch_points(touch_points);

    let modifier_flags = input.modifiers.to_cdp_flags();
    if modifier_flags != 0 {
        params = params.modifiers(modifier_flags);
    }

    let built = params.build().map_err(|e| AgentError::Internal {
        message: format!("Failed to build touch params: {}", e),
    })?;

    page.execute(built)
        .await
        .map_err(|e| AgentError::Internal {
            message: format!("Failed to dispatch touch event: {}", e),
        })?;

    Ok(())
}
