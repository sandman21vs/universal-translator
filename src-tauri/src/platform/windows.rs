// src-tauri/src/platform/windows.rs
use super::*;
use ::windows::Win32::{
    Foundation::HWND,
    System::{
        Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED, SAFEARRAY,
        },
        Ole::{
            SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetElement, SafeArrayGetLBound,
            SafeArrayGetUBound,
        },
    },
    UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
use std::{mem::size_of, time::Duration};
pub struct NativeDesktop;
struct ComScope;
impl ComScope {
    fn new() -> AppResult<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|_| "Acessibilidade COM indisponível.")?;
        }
        Ok(Self)
    }
}
impl Drop for ComScope {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
struct Array(*mut SAFEARRAY);
impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            let _ = SafeArrayDestroy(self.0);
        }
    }
}
fn automation() -> AppResult<IUIAutomation> {
    unsafe {
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            .map_err(|_| "UI Automation indisponível.".into())
    }
}
fn runtime_id(element: &IUIAutomationElement) -> AppResult<Vec<i32>> {
    unsafe {
        let array = Array(
            element
                .GetRuntimeId()
                .map_err(|_| "Elemento não identificável por acessibilidade.")?,
        );
        if array.0.is_null() || SafeArrayGetDim(array.0) != 1 {
            return Err("Identificador de elemento inválido.".into());
        }
        let low = SafeArrayGetLBound(array.0, 1).map_err(|_| "Elemento inválido.")?;
        let high = SafeArrayGetUBound(array.0, 1).map_err(|_| "Elemento inválido.")?;
        if high < low || high - low > 64 {
            return Err("Identificador de elemento inválido.".into());
        }
        let mut id = Vec::new();
        for index in low..=high {
            let mut value = 0i32;
            SafeArrayGetElement(array.0, &index, (&mut value as *mut i32).cast())
                .map_err(|_| "Elemento inválido.")?;
            id.push(value);
        }
        Ok(id)
    }
}
fn safe_element(element: &IUIAutomationElement) -> AppResult<()> {
    unsafe {
        if element
            .CurrentIsPassword()
            .map_err(|_| "Não foi possível verificar segurança do campo.")?
            .as_bool()
        {
            return Err("Campos de senha não são suportados.".into());
        }
        if !element
            .CurrentIsEnabled()
            .map_err(|_| "Campo indisponível.")?
            .as_bool()
        {
            return Err("Campo desabilitado.".into());
        }
        Ok(())
    }
}
fn verify_window(target: &Target) -> AppResult<HWND> {
    unsafe {
        let hwnd = HWND(target.window as *mut std::ffi::c_void);
        let mut process = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process));
        if !IsWindow(Some(hwnd)).as_bool() || process != target.process || process == 0 {
            return Err("A janela de destino fechou ou mudou. Copie a tradução.".into());
        }
        Ok(hwnd)
    }
}
fn same_focus(target: &Target, automation: &IUIAutomation) -> AppResult<IUIAutomationElement> {
    unsafe {
        let hwnd = verify_window(target)?;
        if GetForegroundWindow() != hwnd {
            return Err("O foco mudou. Nenhum texto foi inserido; copie a tradução.".into());
        }
        let element = automation
            .GetFocusedElement()
            .map_err(|_| "Campo de destino indisponível.")?;
        safe_element(&element)?;
        if runtime_id(&element)? != target.focus_id {
            return Err("O campo de destino mudou. Nenhum texto foi inserido.".into());
        }
        Ok(element)
    }
}
fn editable(element: &IUIAutomationElement) -> AppResult<()> {
    unsafe {
        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            if pattern
                .CurrentIsReadOnly()
                .map_err(|_| "Estado de edição indisponível.")?
                .as_bool()
            {
                return Err("Campo somente leitura; use copiar.".into());
            }
            return Ok(());
        }
        let pattern: IUIAutomationTextPattern = element
            .GetCurrentPatternAs(UIA_TextPatternId)
            .map_err(|_| "Não foi possível confirmar que o campo é editável. Use copiar.")?;
        let mut attr = pattern
            .DocumentRange()
            .and_then(|r| r.GetAttributeValue(UIA_IsReadOnlyAttributeId))
            .map_err(|_| "Estado de edição indisponível.")?;
        let read_only = if attr.Anonymous.Anonymous.vt == ::windows::Win32::System::Variant::VT_BOOL
        {
            Some(attr.Anonymous.Anonymous.Anonymous.boolVal.0 != 0)
        } else {
            None
        };
        let _ = ::windows::Win32::System::Variant::VariantClear(&mut attr);
        if read_only != Some(false) {
            return Err("Campo somente leitura ou estado de edição incerto. Use copiar.".into());
        }
        Ok(())
    }
}
fn unicode_inputs(text: &str) -> Vec<INPUT> {
    // Unicode packets, including LF, never VK_RETURN. No clipboard transaction.
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .encode_utf16()
        .flat_map(|unit| {
            [false, true].map(|up| INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: unit,
                        dwFlags: if up {
                            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                        } else {
                            KEYEVENTF_UNICODE
                        },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            })
        })
        .collect()
}
impl WindowTargetService for NativeDesktop {
    fn capture() -> AppResult<Target> {
        let _scope = ComScope::new()?;
        let automation = automation()?;
        unsafe {
            let hwnd = GetForegroundWindow();
            let mut process = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process));
            if hwnd.0.is_null() || process == 0 || process == std::process::id() {
                return Err("Abra pelo atalho sobre um aplicativo externo.".into());
            }
            let element = automation
                .GetFocusedElement()
                .map_err(|_| "Campo com foco indisponível.")?;
            safe_element(&element)?;
            let target = Target {
                window: hwnd.0 as isize,
                process,
                focus_id: runtime_id(&element)?,
            };
            same_focus(&target, &automation)?;
            Ok(target)
        }
    }
}
impl SelectionService for NativeDesktop {
    fn selection(target: &Target) -> AppResult<String> {
        let _scope = ComScope::new()?;
        let automation = automation()?;
        let element = same_focus(target, &automation)?;
        unsafe {
            let pattern: IUIAutomationTextPattern = element
                .GetCurrentPatternAs(UIA_TextPatternId)
                .map_err(|_| {
                    "Aplicativo não expõe seleção por UI Automation. Não usamos clipboard antigo."
                })?;
            let ranges = pattern
                .GetSelection()
                .map_err(|_| "Seleção indisponível.")?;
            if ranges.Length().map_err(|_| "Seleção indisponível.")? != 1 {
                return Err("Seleção vazia ou múltipla não suportada.".into());
            }
            let text = ranges
                .GetElement(0)
                .and_then(|r| r.GetText(16001))
                .map_err(|_| "Texto selecionado indisponível.")?
                .to_string();
            if text.trim().is_empty() {
                return Err("Nenhum texto selecionado.".into());
            }
            if text.chars().count() > 8000 {
                return Err("Seleção excede 8.000 caracteres.".into());
            }
            same_focus(target, &automation)?;
            Ok(text)
        }
    }
}
impl TextInsertionService for NativeDesktop {
    fn insert(target: &Target, text: &str) -> AppResult<()> {
        let _scope = ComScope::new()?;
        let automation = automation()?;
        let hwnd = verify_window(target)?;
        // No AttachThreadInput / arbitrary foreground app / clipboard / Enter key.
        unsafe {
            if !SetForegroundWindow(hwnd).as_bool() {
                return Err(
                    "Windows recusou restaurar o foco. Use copiar; não execute como administrador."
                        .into(),
                );
            }
        }
        for _ in 0..20 {
            unsafe {
                if GetForegroundWindow() == hwnd {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let mut released = false;
        for _ in 0..50 {
            let down = unsafe {
                [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN, VK_RETURN]
                    .into_iter()
                    .any(|key| GetAsyncKeyState(key.0 as i32) < 0)
            };
            if !down {
                released = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if !released {
            return Err("Solte as teclas antes de inserir. Tradução preservada.".into());
        }
        let element = same_focus(target, &automation)?;
        editable(&element)?;
        let inputs = unicode_inputs(text);
        same_focus(target, &automation)?;
        let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            return Err("Inserção bloqueada ou parcial (possível aplicativo elevado). Verifique o destino antes de tentar novamente. Não há retry automático.".into());
        }
        Ok(())
    }
}
pub fn cursor_position() -> Option<(i32, i32)> {
    let mut p = ::windows::Win32::Foundation::POINT::default();
    unsafe { GetCursorPos(&mut p).ok().map(|_| (p.x, p.y)) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_preserves_surrogates_and_no_enter_key() {
        let inputs = unicode_inputs("ação 😊\nESP32-P4");
        assert_eq!(inputs.len(), "ação 😊\nESP32-P4".encode_utf16().count() * 2);
        for input in inputs {
            unsafe {
                assert_eq!(input.Anonymous.ki.wVk, VIRTUAL_KEY(0));
                assert_eq!(
                    input.Anonymous.ki.dwFlags.0 & KEYEVENTF_UNICODE.0,
                    KEYEVENTF_UNICODE.0
                );
            }
        }
    }
    #[test]
    fn closed_target_fails_before_input() {
        let target = Target {
            window: 0,
            process: 123,
            focus_id: vec![],
        };
        assert!(verify_window(&target).is_err());
    }
}
