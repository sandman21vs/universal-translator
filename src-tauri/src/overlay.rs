use crate::core::AppResult;
#[cfg(target_os = "macos")]
use tauri::LogicalPosition;
use tauri::{window::Color, LogicalSize, Manager};
#[cfg(not(target_os = "macos"))]
use tauri::{PhysicalPosition, PhysicalSize};

pub const WIDTH: f64 = 460.0;
pub const HEIGHT: f64 = 340.0;

pub fn configure(app: &tauri::AppHandle, compact: bool) -> AppResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or("Janela indisponível.")?;
    let update = || -> tauri::Result<()> {
        window.hide()?;
        window.unmaximize()?;
        // Remove old constraints first: the full window has a larger minimum.
        window.set_min_size(None::<LogicalSize<f64>>)?;
        window.set_max_size(None::<LogicalSize<f64>>)?;
        window.set_decorations(!compact)?;
        window.set_always_on_top(compact)?;
        window.set_skip_taskbar(compact)?;
        window.set_maximizable(!compact)?;
        window.set_shadow(true)?;
        window.set_background_color(Some(if compact && cfg!(windows) {
            Color(0, 0, 0, 0)
        } else {
            Color(16, 22, 28, 255)
        }))?;
        if compact {
            window.set_size(LogicalSize::new(WIDTH, HEIGHT))?;
            window.set_min_size(Some(LogicalSize::new(380.0, 280.0)))?;
            window.set_max_size(Some(LogicalSize::new(640.0, 480.0)))?;
        } else {
            window.set_size(LogicalSize::new(820.0, 740.0))?;
            window.set_min_size(Some(LogicalSize::new(540.0, 580.0)))?;
            window.center()?;
        }
        Ok(())
    };
    update().map_err(|_| "Não foi possível ajustar a janela do tradutor.".into())
}

// macOS remembers the Space where a hidden window was last shown and switches to it when the
// app activates. Joining every Space and ordering the window in before activation keeps the
// user on the Space where they are typing.
#[cfg(target_os = "macos")]
pub fn bring_to_active_space(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let handle = window.clone();
    let _ = window.run_on_main_thread(move || {
        let Ok(pointer) = handle.ns_window() else {
            return;
        };
        // SAFETY: Tauri owns this NSWindow for as long as `handle` lives, and AppKit is only
        // touched from the main thread.
        let ns_window: &NSWindow = unsafe { &*pointer.cast() };
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
        ns_window.orderFrontRegardless();
    });
}

fn bounds(
    cursor: (i32, i32),
    origin: (i32, i32),
    available: (u32, u32),
    scale: f64,
) -> ((i32, i32), (u32, u32)) {
    let width = ((WIDTH * scale).round() as u32).min(available.0);
    let height = ((HEIGHT * scale).round() as u32).min(available.1);
    let left = (cursor.0 + 16).clamp(origin.0, origin.0 + (available.0 - width) as i32);
    let top = (cursor.1 + 16).clamp(origin.1, origin.1 + (available.1 - height) as i32);
    ((left, top), (width, height))
}

#[cfg(not(target_os = "macos"))]
pub fn position(app: &tauri::AppHandle) {
    let (Some((x, y)), Some(window)) = (
        crate::platform::cursor_position(),
        app.get_webview_window("main"),
    ) else {
        return;
    };
    let Ok(monitors) = window.available_monitors() else {
        return;
    };
    let Some(monitor) = monitors.iter().find(|m| {
        let p = m.position();
        let s = m.size();
        x >= p.x && y >= p.y && x < p.x + s.width as i32 && y < p.y + s.height as i32
    }) else {
        return;
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let ((left, top), (width, height)) = bounds(
        (x, y),
        (area.position.x, area.position.y),
        (area.size.width, area.size.height),
        scale,
    );
    let _ = window.set_min_size(Some(LogicalSize::new(
        380.0_f64.min(width as f64 / scale),
        280.0_f64.min(height as f64 / scale),
    )));
    let _ = window.set_size(PhysicalSize::new(width, height));
    let _ = window.set_position(PhysicalPosition::new(left, top));
}

/// Physical value reported at `scale` back to points, the unit macOS shares across displays.
#[cfg(target_os = "macos")]
fn points(value: f64, scale: f64) -> f64 {
    value / scale
}

// The runtime scales the cursor by the primary display and each monitor by its own factor,
// so mixed-DPI setups only line up once everything is converted to points.
#[cfg(target_os = "macos")]
pub fn position(app: &tauri::AppHandle) {
    let (Ok(cursor), Ok(Some(primary)), Some(window)) = (
        app.cursor_position(),
        app.primary_monitor(),
        app.get_webview_window("main"),
    ) else {
        return;
    };
    let x = points(cursor.x, primary.scale_factor());
    let y = points(cursor.y, primary.scale_factor());
    let Ok(monitors) = window.available_monitors() else {
        return;
    };
    let Some(monitor) = monitors.iter().find(|m| {
        let scale = m.scale_factor();
        let left = points(m.position().x as f64, scale);
        let top = points(m.position().y as f64, scale);
        x >= left
            && y >= top
            && x < left + points(m.size().width as f64, scale)
            && y < top + points(m.size().height as f64, scale)
    }) else {
        return;
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let ((left, top), (width, height)) = bounds(
        (x.round() as i32, y.round() as i32),
        (
            points(area.position.x as f64, scale).round() as i32,
            points(area.position.y as f64, scale).round() as i32,
        ),
        (
            points(area.size.width as f64, scale) as u32,
            points(area.size.height as f64, scale) as u32,
        ),
        1.0,
    );
    let _ = window.set_min_size(Some(LogicalSize::new(
        380.0_f64.min(width as f64),
        280.0_f64.min(height as f64),
    )));
    let _ = window.set_size(LogicalSize::new(width as f64, height as f64));
    let _ = window.set_position(LogicalPosition::new(left as f64, top as f64));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fits_negative_monitor_coordinates_and_dpi_without_covering_taskbar() {
        let ((x, y), (w, h)) = bounds((-5, 1040), (-1920, 0), (1920, 1040), 1.5);
        assert_eq!((w, h), (690, 510));
        assert!(x >= -1920 && x + w as i32 <= 0);
        assert!(y >= 0 && y + h as i32 <= 1040);
        let ((x, y), (w, h)) = bounds((310, 220), (0, 0), (320, 240), 2.0);
        assert_eq!(((x, y), (w, h)), ((0, 0), (320, 240)));
    }
}
