// src-tauri/src/platform/mod.rs
use crate::core::AppResult;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::cursor_position;
#[cfg(windows)]
pub use windows::NativeDesktop;
#[cfg(not(windows))]
pub fn cursor_position() -> Option<(i32, i32)> {
    None
}

#[derive(Clone, Debug)]
#[cfg_attr(not(windows), allow(dead_code))]
pub struct Target {
    pub window: isize,
    pub process: u32,
    pub focus_id: Vec<i32>,
}
pub trait WindowTargetService {
    fn capture() -> AppResult<Target>;
}
pub trait SelectionService {
    fn selection(target: &Target) -> AppResult<String>;
}
pub trait TextInsertionService {
    fn insert(target: &Target, text: &str) -> AppResult<()>;
}

#[cfg(not(windows))]
pub struct NativeDesktop;
#[cfg(not(windows))]
impl WindowTargetService for NativeDesktop {
    fn capture() -> AppResult<Target> {
        Err("Inserção global não implementada nesta plataforma. Traduza e copie.".into())
    }
}
#[cfg(not(windows))]
impl SelectionService for NativeDesktop {
    fn selection(_: &Target) -> AppResult<String> {
        Err("Seleção global não implementada nesta plataforma.".into())
    }
}
#[cfg(not(windows))]
impl TextInsertionService for NativeDesktop {
    fn insert(_: &Target, _: &str) -> AppResult<()> {
        Err("Inserção global não implementada nesta plataforma.".into())
    }
}
