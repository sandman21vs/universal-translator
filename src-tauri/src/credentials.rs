// src-tauri/src/credentials.rs
use crate::core::{AppResult, Config};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub trait CredentialStore {
    fn read(&self, config: &Config) -> AppResult<Option<String>>;
    fn write(&mut self, config: &Config, secret: String, persist: bool) -> AppResult<()>;
}
#[derive(Default)]
pub struct Credentials {
    session: HashMap<String, String>,
}
fn scope(config: &Config) -> String {
    format!(
        "{}:{:x}",
        config.provider,
        Sha256::digest(
            config.providers[&config.provider]
                .base_url
                .trim_end_matches('/')
                .as_bytes()
        )
    )
}
fn entry(config: &Config) -> AppResult<keyring::Entry> {
    keyring::Entry::new(env!("APP_ID"), &scope(config))
        .map_err(|_| "Cofre do sistema indisponível. Use credencial somente nesta sessão.".into())
}
impl CredentialStore for Credentials {
    fn read(&self, config: &Config) -> AppResult<Option<String>> {
        if let Some(key) = self.session.get(&scope(config)) {
            return Ok(Some(key.clone()));
        }
        match entry(config)?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err("Cofre indisponível. Informe uma credencial de sessão.".into()),
        }
    }
    fn write(&mut self, config: &Config, secret: String, persist: bool) -> AppResult<()> {
        if secret.len() > 4096 || secret.trim().is_empty() {
            return Err("Credencial vazia ou inválida.".into());
        }
        if persist {
            entry(config)?.set_password(&secret).map_err(|_| {
                "Falha ao salvar no cofre. Escolha sessão; não houve fallback para arquivo."
                    .to_string()
            })?;
            self.session.remove(&scope(config));
        } else {
            self.session.insert(scope(config), secret);
        }
        Ok(())
    }
}
impl Credentials {
    pub fn remove(&mut self, config: &Config) -> AppResult<()> {
        self.session.remove(&scope(config));
        match entry(config)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Falha ao remover credencial do cofre.".into()),
        }
    }
    pub fn clear_session(&mut self) {
        self.session.clear();
    }
}
