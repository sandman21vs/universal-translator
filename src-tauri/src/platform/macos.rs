// src-tauri/src/platform/macos.rs
use super::*;
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};
use std::time::Duration;

const COMBINED_SESSION_STATE: i32 = 0;
const RETURN_KEY: u16 = 36;
// Shift, Control, Option and Command bits of CGEventFlags.
const MODIFIERS: u64 = 0x0002_0000 | 0x0004_0000 | 0x0008_0000 | 0x0010_0000;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    fn CGEventSourceFlagsState(state: i32) -> u64;
    fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
}
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn IsSecureEventInputEnabled() -> u8;
}

pub struct NativeDesktop;

fn frontmost() -> Option<u32> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    u32::try_from(app.processIdentifier()).ok()
}
/// Asks macOS for permission to type into other apps; the system shows its own dialog once.
fn trusted() -> bool {
    unsafe {
        let prompt = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let options = CFDictionary::from_CFType_pairs(&[(
            prompt.as_CFType(),
            CFBoolean::true_value().as_CFType(),
        )]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) != 0
    }
}
/// macOS turns secure input on while a password field has focus.
fn secure_input() -> bool {
    unsafe { IsSecureEventInputEnabled() != 0 }
}
fn keys_released() -> bool {
    unsafe {
        CGEventSourceFlagsState(COMBINED_SESSION_STATE) & MODIFIERS == 0
            && !CGEventSourceKeyState(COMBINED_SESSION_STATE, RETURN_KEY)
    }
}
fn key(source: &CGEventSource, code: u16, down: bool) -> AppResult<CGEvent> {
    CGEvent::new_keyboard_event(source.clone(), code, down)
        .map_err(|_| "Não foi possível gerar o texto para inserção.".into())
}
/// One key press per character; line breaks become Shift+Return so chat apps do not send.
fn type_text(text: &str) -> AppResult<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Não foi possível gerar o texto para inserção.")?;
    let mut units = [0u16; 2];
    for character in text.chars().filter(|c| *c != '\r') {
        for down in [true, false] {
            let event = if character == '\n' {
                let event = key(&source, RETURN_KEY, down)?;
                event.set_flags(CGEventFlags::CGEventFlagShift);
                event
            } else {
                let event = key(&source, 0, down)?;
                event.set_flags(CGEventFlags::CGEventFlagNull);
                event.set_string_from_utf16_unchecked(character.encode_utf16(&mut units));
                event
            };
            event.post(CGEventTapLocation::HID);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

impl WindowTargetService for NativeDesktop {
    fn capture() -> AppResult<Target> {
        let process = frontmost()
            .filter(|pid| *pid != std::process::id())
            .ok_or("Nenhum aplicativo de destino em primeiro plano.")?;
        if secure_input() {
            return Err("Campos de senha não são suportados.".into());
        }
        Ok(Target {
            window: 0,
            process,
            focus_id: Vec::new(),
        })
    }
}
impl SelectionService for NativeDesktop {
    fn selection(_: &Target) -> AppResult<String> {
        Err("Seleção global não implementada nesta plataforma.".into())
    }
}
impl TextInsertionService for NativeDesktop {
    fn insert(target: &Target, text: &str) -> AppResult<()> {
        if !trusted() {
            return Err("Autorize o Universal Translator em Ajustes do Sistema > Privacidade e Segurança > Acessibilidade e tente de novo. Tradução preservada.".into());
        }
        // No clipboard and no plain Return key: the message is typed, never sent.
        let app = i32::try_from(target.process)
            .ok()
            .and_then(NSRunningApplication::runningApplicationWithProcessIdentifier)
            .ok_or("O aplicativo de destino fechou. Copie a tradução.")?;
        app.activateWithOptions(NSApplicationActivationOptions::empty());
        let mut focused = false;
        for _ in 0..50 {
            if frontmost() == Some(target.process) {
                focused = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if !focused {
            return Err(
                "macOS recusou restaurar o foco. Nenhum texto foi inserido; copie a tradução."
                    .into(),
            );
        }
        let mut released = false;
        for _ in 0..50 {
            if keys_released() {
                released = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if !released {
            return Err("Solte as teclas antes de inserir. Tradução preservada.".into());
        }
        // Give the target window time to become key again before typing into it.
        std::thread::sleep(Duration::from_millis(80));
        if secure_input() {
            return Err("Campos de senha não são suportados.".into());
        }
        if frontmost() != Some(target.process) {
            return Err("O foco mudou. Nenhum texto foi inserido; copie a tradução.".into());
        }
        type_text(text)
    }
}
