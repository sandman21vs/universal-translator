// src-tauri/src/lib.rs
mod core;
mod credentials;
mod local_engine;
mod overlay;
mod platform;
mod providers;

use core::{AppResult, Cache, Config, TranslationRequest, TranslationResult};
use credentials::{CredentialStore, Credentials};
use platform::{
    NativeDesktop, SelectionService, Target, TextInsertionService, WindowTargetService,
};
use providers::{HttpProvider, TranslationProvider};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    time::Instant,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tokio_util::sync::CancellationToken;

struct Pending {
    latest: u64,
    token: CancellationToken,
    ready: Option<u64>,
    busy: bool,
}
impl Default for Pending {
    fn default() -> Self {
        Self {
            latest: 0,
            token: CancellationToken::new(),
            ready: None,
            busy: false,
        }
    }
}
struct AppState {
    compact: AtomicBool,
    revision: AtomicU64,
    config: Mutex<Config>,
    config_path: PathBuf,
    credentials: Mutex<Credentials>,
    cache: Mutex<Cache>,
    pending: Mutex<Pending>,
    target: Mutex<Option<Target>>,
    http: HttpProvider,
    local: local_engine::LocalEngine,
    warning: Mutex<Option<String>>,
    shortcut_error: Mutex<Option<String>>,
    paused: Mutex<bool>,
}
fn lock<T>(mutex: &Mutex<T>) -> AppResult<std::sync::MutexGuard<'_, T>> {
    mutex
        .lock()
        .map_err(|_| "Estado interno indisponível. Reinicie o app.".into())
}
fn invalidate(state: &AppState) -> AppResult<()> {
    let mut p = lock(&state.pending)?;
    state.revision.fetch_add(1, Ordering::SeqCst);
    p.token.cancel();
    p.ready = None;
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Bootstrap {
    compact: bool,
    config: Config,
    warning: Option<String>,
    shortcut_error: Option<String>,
    platform: String,
    insertion_available: bool,
    capabilities: providers::Capabilities,
    paused: bool,
}
#[tauri::command]
fn bootstrap(state: State<AppState>) -> AppResult<Bootstrap> {
    let config = lock(&state.config)?.clone();
    Ok(Bootstrap {
        compact: state.compact.load(Ordering::SeqCst),
        capabilities: providers::capabilities(&config.provider),
        config,
        warning: lock(&state.warning)?.clone(),
        shortcut_error: lock(&state.shortcut_error)?.clone(),
        platform: std::env::consts::OS.into(),
        insertion_available: lock(&state.target)?.is_some(),
        paused: *lock(&state.paused)?,
    })
}
#[tauri::command]
async fn translate(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: TranslationRequest,
) -> AppResult<TranslationResult> {
    request.validate()?;
    let revision = state.revision.load(Ordering::SeqCst);
    let config = lock(&state.config)?.clone();
    config.validate()?;
    if config.provider != "local" {
        core::endpoint(&config)?;
    }
    let token = {
        let mut p = lock(&state.pending)?;
        if request.id <= p.latest || p.busy || revision != state.revision.load(Ordering::SeqCst) {
            return Err("Requisição obsoleta ou inserção em andamento.".into());
        }
        p.token.cancel();
        p.latest = request.id;
        p.ready = None;
        p.token = CancellationToken::new();
        p.token.clone()
    };
    let start = Instant::now();
    let cache_key = Cache::key(&config, &request);
    let cached = lock(&state.cache)?.get(&cache_key);
    let (text, was_cached) = if let Some(text) = cached {
        (text, true)
    } else {
        let key = if matches!(config.provider.as_str(), "ollama" | "local") {
            None
        } else {
            read_secret(app, config.clone()).await?
        };
        let translate = async {
            if config.provider == "local" {
                state.local.translate(&config, &request).await
            } else {
                state
                    .http
                    .translate(&config, &request, key.as_deref())
                    .await
            }
        };
        let text = tokio::select! { biased; _=token.cancelled()=>return Err("Tradução cancelada.".into()), result=tokio::time::timeout(std::time::Duration::from_secs(config.timeout_secs),translate)=>result.map_err(|_|"Tempo total de tradução esgotado.".to_string())?? };
        (text, false)
    };
    {
        let mut p = lock(&state.pending)?;
        if p.latest != request.id
            || token.is_cancelled()
            || revision != state.revision.load(Ordering::SeqCst)
        {
            return Err("Resposta obsoleta descartada.".into());
        }
        p.ready = Some(request.id);
        if !was_cached {
            lock(&state.cache)?.put(cache_key, text.clone());
        }
    }
    Ok(TranslationResult {
        id: request.id,
        text,
        provider: config.provider.clone(),
        model: config.providers[&config.provider].model.clone(),
        elapsed_ms: start.elapsed().as_millis(),
        cached: was_cached,
    })
}
#[tauri::command]
fn cancel_translation(state: State<AppState>, id: u64) -> AppResult<()> {
    let mut p = lock(&state.pending)?;
    if id > p.latest {
        p.latest = id;
        p.token.cancel();
        p.ready = None;
    }
    Ok(())
}
fn register_shortcuts(app: &tauri::AppHandle, config: &Config) -> AppResult<()> {
    let compose: Shortcut = config
        .compose_shortcut
        .parse()
        .map_err(|_| "Atalho de composição inválido.")?;
    let selection: Shortcut = config
        .selection_shortcut
        .parse()
        .map_err(|_| "Atalho de seleção inválido.")?;
    let repeat: Shortcut = config
        .repeat_shortcut
        .parse()
        .map_err(|_| "Atalho de repetição inválido.")?;
    app.global_shortcut()
        .unregister_all()
        .map_err(|_| "Falha ao remover atalhos anteriores.")?;
    if let Err(_e) = app
        .global_shortcut()
        .register_multiple([compose, selection, repeat])
    {
        let _ = app.global_shortcut().unregister_all();
        return Err("Não foi possível registrar os atalhos. Verifique conflitos e AltGr; nenhum atalho foi anunciado como ativo.".into());
    }
    Ok(())
}
#[tauri::command]
async fn save_settings(app: tauri::AppHandle, config: Config) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        save_settings_inner(app.clone(), &state, config)
    })
    .await
    .map_err(|_| "Falha ao salvar configurações.".to_string())?
}
fn save_settings_inner(app: tauri::AppHandle, state: &AppState, config: Config) -> AppResult<()> {
    config.validate()?;
    for s in [
        &config.compose_shortcut,
        &config.selection_shortcut,
        &config.repeat_shortcut,
    ] {
        let _: Shortcut = s.parse().map_err(|_| "Atalho inválido.")?;
    }
    let old_file = std::fs::read(&state.config_path).ok();
    core::save_config(&state.config_path, &config)?;
    if let Err(e) = if config.autostart {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    } {
        if let Some(bytes) = old_file {
            let _ = std::fs::write(&state.config_path, bytes);
        } else {
            let _ = std::fs::remove_file(&state.config_path);
        }
        return Err(format!(
            "Falha ao configurar início automático ({:?}). Configuração anterior preservada.",
            e
        ));
    }
    {
        let mut saved = lock(&state.config)?;
        invalidate(state)?;
        lock(&state.cache)?.clear();
        *saved = config.clone();
    }
    if !*lock(&state.paused)? {
        *lock(&state.shortcut_error)? = register_shortcuts(&app, &config).err();
    }
    Ok(())
}
#[tauri::command]
async fn set_credential(app: tauri::AppHandle, secret: String, persist: bool) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        set_credential_inner(&state, secret, persist)
    })
    .await
    .map_err(|_| "Falha ao salvar credencial.".to_string())?
}
fn set_credential_inner(state: &AppState, secret: String, persist: bool) -> AppResult<()> {
    let c = lock(&state.config)?.clone();
    lock(&state.credentials)?.write(&c, secret, persist)?;
    invalidate(state)?;
    lock(&state.cache)?.clear();
    Ok(())
}
#[tauri::command]
async fn remove_credential(app: tauri::AppHandle) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        remove_credential_inner(&state)
    })
    .await
    .map_err(|_| "Falha ao remover credencial.".to_string())?
}
fn remove_credential_inner(state: &AppState) -> AppResult<()> {
    let c = lock(&state.config)?.clone();
    lock(&state.credentials)?.remove(&c)?;
    invalidate(state)?;
    lock(&state.cache)?.clear();
    Ok(())
}
#[tauri::command]
async fn inspect_provider(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<providers::Inspection> {
    let config = lock(&state.config)?.clone();
    let key = if matches!(config.provider.as_str(), "ollama" | "local") {
        None
    } else {
        read_secret(app, config.clone()).await?
    };
    if config.provider == "local" {
        let start = Instant::now();
        let status = state.local.status().await?;
        let items = status["models"]
            .as_array()
            .ok_or("Estado local inválido.")?
            .iter()
            .filter(|m| m["installed"] == true)
            .filter_map(|m| m["id"].as_str().map(str::to_string))
            .collect();
        Ok(providers::Inspection {
            items,
            kind: "languages".into(),
            elapsed_ms: start.elapsed().as_millis(),
        })
    } else {
        state.http.inspect(&config, key.as_deref()).await
    }
}
#[tauri::command]
async fn local_engine_status(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    state.local.status().await
}
#[tauri::command]
async fn install_local_model(state: State<'_, AppState>, model_id: String) -> AppResult<()> {
    state.local.install(&model_id).await?;
    invalidate(&state)?;
    lock(&state.cache)?.clear();
    Ok(())
}
async fn read_secret(app: tauri::AppHandle, config: Config) -> AppResult<Option<String>> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = lock(&state.credentials)?.read(&config);
        result
    })
    .await
    .map_err(|_| "Falha ao acessar credenciais.".to_string())?
}
#[tauri::command]
async fn copy_text(text: String) -> AppResult<()> {
    if text.len() > 64000 {
        return Err("Texto longo demais.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        arboard::Clipboard::new()
            .and_then(|mut c| c.set_text(text))
            .map_err(|_| "Clipboard indisponível.".into())
    })
    .await
    .map_err(|_| "Falha ao copiar.")?
}
#[tauri::command]
async fn insert_translation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: u64,
    text: String,
) -> AppResult<()> {
    if text.trim().is_empty() || text.len() > 64000 {
        return Err("Tradução vazia ou longa demais.".into());
    }
    let target = lock(&state.target)?.clone().ok_or(
        "Sem destino registrado. Use o atalho sobre o campo de destino ou copie a tradução.",
    )?;
    {
        let mut p = lock(&state.pending)?;
        if p.ready != Some(id) || p.token.is_cancelled() || p.busy {
            return Err("Tradução obsoleta ou incompleta; aguarde o resultado atual.".into());
        }
        p.busy = true;
        p.ready = None;
    }
    let outcome =
        tauri::async_runtime::spawn_blocking(move || NativeDesktop::insert(&target, &text))
            .await
            .map_err(|_| "Falha no adaptador desktop.".to_string())
            .and_then(|r| r);
    lock(&state.pending)?.busy = false;
    if outcome.is_ok() {
        *lock(&state.target)? = None;
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
    } else if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
    outcome
}
#[tauri::command]
fn hide_overlay(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    invalidate(&state)?;
    *lock(&state.target)? = None;
    app.get_webview_window("main")
        .ok_or("Janela indisponível.")?
        .hide()
        .map_err(|_| "Falha ao ocultar janela.")?;
    // Hand keyboard focus back to the app the user was typing in.
    #[cfg(target_os = "macos")]
    let _ = app.hide();
    Ok(())
}
#[tauri::command]
fn clear_session(state: State<AppState>) -> AppResult<()> {
    invalidate(&state)?;
    lock(&state.cache)?.clear();
    lock(&state.credentials)?.clear_session();
    *lock(&state.target)? = None;
    Ok(())
}
#[tauri::command]
fn open_settings(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    invalidate(&state)?;
    *lock(&state.target)? = None;
    show(&app, "settings");
    Ok(())
}
#[tauri::command]
fn drag_overlay(window: tauri::WebviewWindow) -> AppResult<()> {
    window
        .start_dragging()
        .map_err(|_| "Não foi possível mover o painel.".into())
}
fn show(app: &tauri::AppHandle, event: &str) {
    let compact = event != "settings";
    // A Dock app cannot draw over another app's full-screen Space; an accessory (tray) app can.
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(if compact {
        tauri::ActivationPolicy::Accessory
    } else {
        tauri::ActivationPolicy::Regular
    });
    if let Err(error) = overlay::configure(app, compact) {
        let _ = app.emit("desktop-error", error);
    } else {
        app.state::<AppState>()
            .compact
            .store(compact, Ordering::SeqCst);
    }
    if compact {
        overlay::position(app);
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.emit(event, ());
        #[cfg(target_os = "macos")]
        {
            let _ = app.show();
            overlay::bring_to_active_space(&w);
        }
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
fn open_compose(app: &tauri::AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let target = tauri::async_runtime::spawn_blocking(NativeDesktop::capture).await;
        // Pressing the shortcut again over the open panel must not drop the captured target.
        let open = handle
            .get_webview_window("main")
            .and_then(|w| w.is_visible().ok())
            .unwrap_or(false)
            && handle.state::<AppState>().compact.load(Ordering::SeqCst);
        if let Ok(mut saved) = handle.state::<AppState>().target.lock() {
            match target.ok().and_then(Result::ok) {
                Some(target) => *saved = Some(target),
                None if open => {}
                None => *saved = None,
            }
        }
        show(&handle, "compose");
    });
}
fn open_selection(app: &tauri::AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = tauri::async_runtime::spawn_blocking(|| {
            let t = NativeDesktop::capture()?;
            NativeDesktop::selection(&t)
        })
        .await;
        if let Ok(mut target) = handle.state::<AppState>().target.lock() {
            *target = None;
        }
        show(&handle, "selection-open");
        match result {
            Ok(Ok(text)) => {
                let _ = handle.emit("selection", text);
            }
            _ => {
                let _=handle.emit("desktop-error","Não foi possível obter uma seleção por acessibilidade. Selecione texto compatível ou componha e copie. Substituição não disponível nesta versão.");
            }
        }
    });
}
#[tauri::command]
fn pause_shortcuts(app: tauri::AppHandle, state: State<AppState>, paused: bool) -> AppResult<()> {
    if paused {
        app.global_shortcut()
            .unregister_all()
            .map_err(|_| "Falha ao pausar atalhos.")?;
        *lock(&state.shortcut_error)? = None;
    } else {
        let config = lock(&state.config)?.clone();
        *lock(&state.shortcut_error)? = register_shortcuts(&app, &config).err();
    }
    *lock(&state.paused)? = paused;
    Ok(())
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show(app, "compose")
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<AppState>();
                    let Ok(c) = state.config.lock() else {
                        return;
                    };
                    let compose = c.compose_shortcut.parse::<Shortcut>().ok();
                    let selection = c.selection_shortcut.parse::<Shortcut>().ok();
                    let repeat = c.repeat_shortcut.parse::<Shortcut>().ok();
                    drop(c);
                    if compose.as_ref() == Some(shortcut) {
                        open_compose(app);
                    } else if selection.as_ref() == Some(shortcut) {
                        open_selection(app);
                    } else if repeat.as_ref() == Some(shortcut) {
                        let _ = invalidate(&state);
                        if let Ok(mut target) = state.target.lock() {
                            *target = None;
                        }
                        show(app, "repeat");
                    }
                })
                .build(),
        )
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let (config, warning) = core::load_config(&path);
            app.manage(AppState {
                compact: AtomicBool::new(false),
                revision: AtomicU64::new(0),
                config: Mutex::new(config.clone()),
                config_path: path,
                credentials: Mutex::new(Credentials::default()),
                cache: Mutex::new(Cache::default()),
                pending: Mutex::new(Pending::default()),
                target: Mutex::new(None),
                http: HttpProvider::new()?,
                local: local_engine::LocalEngine::new(
                    app.path().app_data_dir()?.join("local-models"),
                )?,
                warning: Mutex::new(warning),
                shortcut_error: Mutex::new(None),
                paused: Mutex::new(false),
            });
            // Native transparency is enabled at creation on Windows and macOS. The
            // full interface paints an opaque background; only the compact panel
            // exposes alpha. Other platforms keep the existing opaque surface.
            tauri::WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
                .transparent(cfg!(any(windows, target_os = "macos")))
                .background_color(tauri::window::Color(0, 0, 0, 0))
                .build()?;
            *app.state::<AppState>()
                .shortcut_error
                .lock()
                .map_err(|_| "state")? = register_shortcuts(app.handle(), &config).err();
            let compose =
                MenuItem::with_id(app, "compose", "Escrever / traduzir", true, None::<&str>)?;
            let selection = MenuItem::with_id(
                app,
                "selection",
                "Traduzir seleção (leitura)",
                true,
                None::<&str>,
            )?;
            let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
            let pause =
                MenuItem::with_id(app, "pause", "Pausar / retomar atalhos", true, None::<&str>)?;
            let clear = MenuItem::with_id(app, "clear", "Limpar sessão", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&compose, &selection, &settings, &pause, &clear, &quit],
            )?;
            let icon = app
                .default_window_icon()
                .ok_or("Ícone indisponível")?
                .clone();
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip(env!("APP_NAME"))
                .menu(&menu)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "compose" => open_compose(app),
                    "selection" => open_selection(app),
                    "settings" => {
                        let _ = open_settings(app.clone(), app.state());
                    }
                    "clear" => {
                        let _ = clear_session(app.state());
                        let _ = app.emit("clear-session", ());
                    }
                    "pause" => {
                        let paused = app
                            .state::<AppState>()
                            .paused
                            .lock()
                            .map(|p| !*p)
                            .unwrap_or(true);
                        let _ = pause_shortcuts(app.clone(), app.state(), paused);
                        let _ = app.emit("shortcuts-changed", ());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = hide_overlay(window.app_handle().clone(), window.state());
            }
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            open_settings,
            drag_overlay,
            translate,
            cancel_translation,
            save_settings,
            set_credential,
            remove_credential,
            inspect_provider,
            local_engine_status,
            install_local_model,
            copy_text,
            insert_translation,
            hide_overlay,
            clear_session,
            pause_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Universal Translator");
}
