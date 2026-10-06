use crate::core::{AppResult, Config, TranslationRequest};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Deserialize)]
struct Model {
    id: String,
    url: String,
    bytes: u64,
    sha256: String,
}

pub struct LocalEngine {
    binary: PathBuf,
    data: PathBuf,
    worker: tokio::sync::Semaphore,
    installation: tokio::sync::Mutex<()>,
}
impl LocalEngine {
    pub fn new(data: PathBuf) -> AppResult<Self> {
        let executable = std::env::current_exe().map_err(|_| "Diretório do app indisponível.")?;
        let name = if cfg!(windows) {
            "local-engine.exe"
        } else {
            "local-engine"
        };
        let binary = executable
            .parent()
            .ok_or("Diretório do app indisponível.")?
            .join(name);
        #[cfg(debug_assertions)]
        let binary = if binary.is_file() {
            binary
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(format!(
                    "local-engine-{}{}",
                    env!("ENGINE_TARGET"),
                    if cfg!(windows) { ".exe" } else { "" }
                ))
        };
        Ok(Self {
            binary,
            data,
            worker: tokio::sync::Semaphore::new(1),
            installation: tokio::sync::Mutex::new(()),
        })
    }
    async fn run(&self, request: Value) -> AppResult<Value> {
        let _permit = self
            .worker
            .acquire()
            .await
            .map_err(|_| "Motor local indisponível.")?;
        if !self.binary.is_file() {
            return Err("Motor local não empacotado. Prepare o sidecar com scripts/build-engine.py ou instale a versão completa do app.".into());
        }
        let mut command = tokio::process::Command::new(&self.binary);
        command
            .arg("--data-dir")
            .arg(&self.data)
            .arg("--managed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW; console build retains stdio.
        let mut child = command
            .spawn()
            .map_err(|_| "Não foi possível iniciar o motor local.")?;
        let mut input = child
            .stdin
            .take()
            .ok_or("Canal do motor local indisponível.")?;
        let mut bytes = serde_json::to_vec(&request).map_err(|_| "Requisição local inválida.")?;
        bytes.push(b'\n');
        input
            .write_all(&bytes)
            .await
            .map_err(|_| "Falha ao enviar dados ao motor local.")?;
        let output = child
            .stdout
            .take()
            .ok_or("Canal do motor local indisponível.")?;
        let mut reply = Vec::new();
        output
            .take(65_537)
            .read_to_end(&mut reply)
            .await
            .map_err(|_| "Falha ao ler resposta local.")?;
        if reply.len() > 65_536 {
            return Err("Resposta local excede o limite.".into());
        }
        if !child
            .wait()
            .await
            .map_err(|_| "Motor local indisponível.")?
            .success()
        {
            return Err("Motor local terminou inesperadamente.".into());
        }
        drop(input);
        let value: Value =
            serde_json::from_slice(&reply).map_err(|_| "Resposta do motor local inválida.")?;
        if value["ok"] != true {
            return Err(value["error"]
                .as_str()
                .unwrap_or("Falha no motor local.")
                .to_string());
        }
        Ok(value["data"].clone())
    }
    pub async fn status(&self) -> AppResult<Value> {
        tokio::time::timeout(
            Duration::from_secs(45),
            self.run(json!({"action":"status"})),
        )
        .await
        .map_err(|_| "Tempo de inicialização do motor local esgotado.".to_string())?
    }
    pub async fn translate(
        &self,
        config: &Config,
        request: &TranslationRequest,
    ) -> AppResult<String> {
        let result = self.run(json!({"action":"translate", "source":config.source, "target":config.target, "text":request.text})).await?;
        result["text"]
            .as_str()
            .filter(|t| !t.trim().is_empty())
            .map(str::to_string)
            .ok_or("Tradução local vazia.".into())
    }
    pub async fn install(&self, model_id: &str) -> AppResult<()> {
        let _installation = self
            .installation
            .try_lock()
            .map_err(|_| "Já existe uma instalação de modelo em andamento.")?;
        let models: Vec<Model> = serde_json::from_str(include_str!("../../engine/catalog.json"))
            .map_err(|_| "Catálogo local inválido.")?;
        let model = models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or("Modelo fora do catálogo verificado.")?;
        // Dedicated download client. Translation text / credentials never enter this request.
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("UniversalTranslator/0.1 model-installer")
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|_| "Falha ao iniciar download.")?;
        let response = client
            .get(&model.url)
            .send()
            .await
            .map_err(|_| "Falha ao baixar modelo. Verifique a conexão.")?;
        if !response.status().is_success() {
            return Err(format!(
                "Servidor de modelos retornou HTTP {}. Nenhum modelo foi instalado.",
                response.status().as_u16()
            ));
        }
        if response
            .content_length()
            .is_some_and(|size| size != model.bytes)
        {
            return Err("Tamanho do modelo diferente do catálogo verificado.".into());
        }
        std::fs::create_dir_all(&self.data).map_err(|_| "Falha ao criar diretório dos modelos.")?;
        let temporary = tempfile::NamedTempFile::new_in(&self.data)
            .map_err(|_| "Falha ao criar arquivo de download.")?
            .into_temp_path();
        let mut file = tokio::fs::File::create(&temporary)
            .await
            .map_err(|_| "Falha ao gravar modelo.")?;
        let mut stream = response.bytes_stream();
        let mut hash = Sha256::new();
        let mut size = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| "Download do modelo interrompido.")?;
            size += chunk.len() as u64;
            if size > model.bytes {
                return Err("Download excede tamanho verificado.".into());
            }
            hash.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|_| "Falha ao gravar modelo. Verifique o espaço em disco.")?;
        }
        file.sync_all()
            .await
            .map_err(|_| "Falha ao sincronizar modelo.")?;
        drop(file);
        if size != model.bytes || format!("{:x}", hash.finalize()) != model.sha256 {
            return Err("Checksum ou tamanho do modelo inválido. Download descartado.".into());
        }
        tokio::time::timeout(Duration::from_secs(90), self.run(json!({"action":"install", "modelId":model.id, "archive":temporary.to_string_lossy()}))).await.map_err(|_| "Tempo de instalação do modelo esgotado.".to_string())??;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn rejects_unknown_models_before_network_or_process() {
        let directory = tempfile::tempdir().unwrap();
        let engine = LocalEngine::new(directory.path().join("models")).unwrap();
        assert!(engine
            .install("../../arbitrary")
            .await
            .unwrap_err()
            .contains("catálogo"));
        assert!(!directory.path().join("models").exists());
    }
    #[tokio::test]
    #[ignore = "Explicit download and real packaged worker: UT_INTEGRATION_LOCAL_ENGINE=1"]
    async fn real_local_download_and_translation() {
        assert_eq!(
            std::env::var("UT_INTEGRATION_LOCAL_ENGINE").as_deref(),
            Ok("1")
        );
        let directory = tempfile::tempdir().unwrap();
        let engine = LocalEngine::new(directory.path().join("models")).unwrap();
        engine.install("pt-en-1.9").await.unwrap();
        let status = engine.status().await.unwrap();
        assert!(status["models"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["id"] == "pt-en-1.9" && m["installed"] == true));
        let request = TranslationRequest {
            id: 1,
            text: "Bom dia!\nObrigado pela ajuda.".into(),
            operation: "translate".into(),
        };
        let start = std::time::Instant::now();
        let result = engine
            .translate(&Config::default(), &request)
            .await
            .unwrap();
        assert!(result.to_lowercase().contains("good morning"));
        assert!(result.to_lowercase().contains("thank"));
        assert!(result.contains('\n'));
        println!(
            "Real packaged local translation: {} ms",
            start.elapsed().as_millis()
        );
    }
}
