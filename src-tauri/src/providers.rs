// src-tauri/src/providers.rs
use crate::core::{endpoint, prompt, AppResult, Config, TranslationRequest};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub styles: bool,
    pub regional_variants: bool,
    pub streaming: bool,
    pub protocol: &'static str,
}
pub fn capabilities(provider: &str) -> Capabilities {
    Capabilities {
        styles: !matches!(provider, "local" | "google" | "libretranslate"),
        regional_variants: !matches!(provider, "local" | "google" | "libretranslate"),
        streaming: false,
        protocol: match provider {
            "local" => "CTranslate2 / modelos Argos · stdio local",
            "ollama" => "Ollama Chat",
            "openai" => "Chat Completions (Responses não implementado)",
            "anthropic" => "Messages 2023-06-01",
            "google" => "Cloud Translation Basic v2 / API key",
            _ => "LibreTranslate HTTP",
        },
    }
}
#[async_trait]
pub trait TranslationProvider {
    async fn translate(
        &self,
        config: &Config,
        request: &TranslationRequest,
        key: Option<&str>,
    ) -> AppResult<String>;
}
pub struct HttpProvider {
    client: reqwest::Client,
    #[cfg(test)]
    test_origin: Option<String>,
}
impl HttpProvider {
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            #[cfg(test)]
            test_origin: None,
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .build()
                .map_err(|_| "Falha ao iniciar HTTP.")?,
        })
    }
    fn url(&self, config: &Config, route: &str) -> AppResult<String> {
        let validated = endpoint(config)?;
        #[cfg(test)]
        if let Some(origin) = &self.test_origin {
            return Ok(format!("{}{}", origin, route));
        }
        Ok(format!(
            "{}{}",
            validated.as_str().trim_end_matches('/'),
            route
        ))
    }
    async fn json(&self, builder: reqwest::RequestBuilder, timeout: u64) -> AppResult<Value> {
        tokio::time::timeout(Duration::from_secs(timeout), async {
            let response = builder.send().await.map_err(|e| {
                if e.is_timeout() {
                    "Tempo de conexão esgotado."
                } else {
                    "Falha de conexão com o provider."
                }
            })?;
            let status = response.status();
            if !status.is_success() {
                return Err(match status.as_u16() {
                    401 | 403 => "Credencial inválida ou acesso negado.",
                    404 => "Endpoint ou modelo não encontrado.",
                    429 => "Limite de requisições ou quota atingido. Tente depois.",
                    500..=599 => "Provider indisponível.",
                    _ => "Provider recusou a requisição.",
                }
                .into());
            }
            let mut bytes = Vec::new();
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|_| "Resposta interrompida.")?;
                if bytes.len() + chunk.len() > 1024 * 1024 {
                    return Err("Resposta excede o limite de segurança.".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            serde_json::from_slice(&bytes).map_err(|_| "Resposta JSON inválida do provider.".into())
        })
        .await
        .map_err(|_| "Tradução excedeu o timeout configurado.".to_string())?
    }
    pub async fn inspect(&self, config: &Config, key: Option<&str>) -> AppResult<Inspection> {
        let start = Instant::now();
        let route = match config.provider.as_str() {
            "ollama" => "/api/tags",
            "libretranslate" => "/languages",
            "openai" => "/models",
            "anthropic" => "/v1/models",
            "google" => "/language/translate/v2/languages",
            _ => return Err("Provider inválido.".into()),
        };
        let mut req = self.client.get(self.url(config, route)?);
        match config.provider.as_str() {
            "openai" => {
                if let Some(k) = key {
                    req = req.bearer_auth(k);
                }
            }
            "anthropic" => {
                req = req
                    .header("x-api-key", key.ok_or("Informe a API key.")?)
                    .header("anthropic-version", "2023-06-01");
            }
            "google" => {
                req = req.header(
                    "x-goog-api-key",
                    key.ok_or("Informe a API key do Google Cloud.")?,
                );
            }
            _ => {}
        }
        let value = self.json(req, config.timeout_secs).await?;
        let items=match config.provider.as_str() {
            "ollama"=>value["models"].as_array().map(|a|a.iter().filter_map(|v|v["name"].as_str().map(String::from)).collect()),
            "libretranslate"=>value.as_array().map(|a|a.iter().filter_map(|v|v["code"].as_str().map(String::from)).collect()),
            "google"=>value["data"]["languages"].as_array().map(|a|a.iter().filter_map(|v|v["language"].as_str().map(String::from)).collect()),
            _=>value["data"].as_array().map(|a|a.iter().filter_map(|v|v["id"].as_str().map(String::from)).collect()),
        }.ok_or("Resposta de descoberta inválida. Alguns endpoints compatíveis não oferecem /models; configure o ID manualmente.")?;
        Ok(Inspection {
            items,
            elapsed_ms: start.elapsed().as_millis(),
            kind: if matches!(config.provider.as_str(), "libretranslate" | "google") {
                "languages"
            } else {
                "models"
            }
            .into(),
        })
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub items: Vec<String>,
    pub elapsed_ms: u128,
    pub kind: String,
}
pub fn payload(config: &Config, request: &TranslationRequest) -> Value {
    let p = &config.providers[&config.provider];
    let system = prompt(config);
    let mut value = match config.provider.as_str() {
        "ollama" => {
            json!({"model":p.model,"messages":[{"role":"system","content":system},{"role":"user","content":request.text}],"stream":false,"think":false,"options":{"temperature":config.temperature,"num_predict":8192}})
        }
        "openai" => {
            json!({"model":p.model,"messages":[{"role":"system","content":system},{"role":"user","content":request.text}],"stream":false,"temperature":config.temperature})
        }
        "anthropic" => {
            json!({"model":p.model,"system":system,"messages":[{"role":"user","content":request.text}],"max_tokens":8192,"temperature":config.temperature,"stream":false})
        }
        "google" => {
            json!({"q":request.text,"target":config.target.split('-').next().unwrap(),"format":"text"})
        }
        _ => {
            json!({"q":request.text,"source":config.source.split('-').next().unwrap(),"target":config.target.split('-').next().unwrap(),"format":"text"})
        }
    };
    if config.provider == "google" && config.source != "auto" {
        value["source"] = json!(config.source.split('-').next().unwrap());
    }
    value
}
pub fn parse(provider: &str, value: &Value) -> AppResult<String> {
    let text = match provider {
        "ollama" => {
            if value["done"] != true || value["done_reason"] == "length" {
                return Err("Tradução incompleta.".into());
            }
            value["message"]["content"].as_str().map(String::from)
        }
        "openai" => {
            if value["choices"][0]["finish_reason"] != "stop" {
                return Err("Tradução incompleta ou recusada.".into());
            }
            value["choices"][0]["message"]["content"]
                .as_str()
                .map(String::from)
        }
        "anthropic" => {
            if value["stop_reason"] != "end_turn" {
                return Err("Tradução incompleta ou recusada.".into());
            }
            value["content"].as_array().map(|a| {
                a.iter()
                    .filter(|v| v["type"] == "text")
                    .filter_map(|v| v["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
        }
        "google" => value["data"]["translations"][0]["translatedText"]
            .as_str()
            .map(|s| html_escape::decode_html_entities(s).into_owned()),
        _ => value["translatedText"].as_str().map(String::from),
    }
    .filter(|s| !s.trim().is_empty() && s.len() <= 64000)
    .ok_or("Provider retornou texto vazio ou inválido.")?;
    Ok(text)
}
#[async_trait]
impl TranslationProvider for HttpProvider {
    async fn translate(
        &self,
        config: &Config,
        request: &TranslationRequest,
        key: Option<&str>,
    ) -> AppResult<String> {
        request.validate()?;
        config.validate()?;
        if capabilities(&config.provider).styles
            && config.providers[&config.provider].model.trim().is_empty()
        {
            return Err("Escolha um modelo nas configurações.".into());
        }
        if config.provider == "ollama" {
            let tags = self
                .json(
                    self.client.get(self.url(config, "/api/tags")?),
                    config.timeout_secs,
                )
                .await?;
            let model = tags["models"]
                .as_array()
                .and_then(|models| {
                    models.iter().find(|m| {
                        m["name"].as_str() == Some(config.providers["ollama"].model.as_str())
                    })
                })
                .ok_or("Modelo não encontrado no Ollama. Liste e escolha um modelo instalado.")?;
            if (model["remote_host"].as_str().is_some_and(|s| !s.is_empty())
                || model["remote_model"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
                || config.providers["ollama"].model.ends_with(":cloud"))
                && !config.providers["ollama"].allow_remote
            {
                return Err("Este modelo Ollama usa cloud mesmo com endpoint local. Autorize envio remoto explicitamente ou escolha um modelo local.".into());
            }
        }
        let route = match config.provider.as_str() {
            "ollama" => "/api/chat",
            "openai" => "/chat/completions",
            "anthropic" => "/v1/messages",
            "google" => "/language/translate/v2",
            _ => "/translate",
        };
        let mut data = payload(config, request);
        let mut req = self.client.post(self.url(config, route)?);
        match config.provider.as_str() {
            "openai" => {
                if let Some(k) = key {
                    req = req.bearer_auth(k);
                }
            }
            "anthropic" => {
                req = req
                    .header("x-api-key", key.ok_or("Informe a API key.")?)
                    .header("anthropic-version", "2023-06-01");
            }
            "google" => {
                req = req.header(
                    "x-goog-api-key",
                    key.ok_or("Informe a API key do Google Cloud.")?,
                );
            }
            "libretranslate" => {
                if let Some(k) = key {
                    data["api_key"] = json!(k);
                }
            }
            _ => {}
        }
        let value = self.json(req.json(&data), config.timeout_secs).await?;
        parse(&config.provider, &value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{body_json, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };
    #[tokio::test]
    async fn google_http_contract_uses_header_and_official_payload() {
        let server = MockServer::start().await;
        let c = Config {
            provider: "google".into(),
            ..Config::default()
        };
        let mut c = c;
        c.providers.get_mut("google").unwrap().allow_remote = true;
        let r = TranslationRequest {
            id: 1,
            text: "ESP32-P4 & GPIO38 😊".into(),
            operation: "translate".into(),
        };
        Mock::given(method("POST"))
            .and(path("/language/translate/v2"))
            .and(header("x-goog-api-key", "test-key"))
            .and(body_json(payload(&c, &r)))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"data":{"translations":[{"translatedText":"ESP32-P4 &amp; GPIO38 😊"}]}}),
            ))
            .expect(1)
            .mount(&server)
            .await;
        let mut provider = HttpProvider::new().unwrap();
        provider.test_origin = Some(server.uri());
        assert_eq!(
            provider.translate(&c, &r, Some("test-key")).await.unwrap(),
            "ESP32-P4 & GPIO38 😊"
        );
    }
    #[tokio::test]
    async fn local_ollama_cloud_model_never_sends_content_without_consent() {
        let server = MockServer::start().await;
        let mut c = Config {
            provider: "ollama".into(),
            ..Config::default()
        };
        c.providers.get_mut("ollama").unwrap().base_url = server.uri();
        c.providers.get_mut("ollama").unwrap().model = "alias".into();
        Mock::given(method("GET")).and(path("/api/tags")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"models":[{"name":"alias","remote_host":"https://ollama.com","remote_model":"cloud-model"}]}))).expect(1).mount(&server).await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .mount(&server)
            .await;
        let r = TranslationRequest {
            id: 1,
            text: "private draft".into(),
            operation: "translate".into(),
        };
        assert!(HttpProvider::new()
            .unwrap()
            .translate(&c, &r, None)
            .await
            .unwrap_err()
            .contains("cloud"));
    }
    #[tokio::test]
    #[ignore = "Requires explicitly selected installed local model; no downloads"]
    async fn installed_local_ollama_opt_in() {
        let model = std::env::var("UT_INTEGRATION_OLLAMA_MODEL")
            .expect("Set UT_INTEGRATION_OLLAMA_MODEL explicitly");
        let mut c = Config {
            provider: "ollama".into(),
            ..Config::default()
        };
        c.providers.get_mut("ollama").unwrap().model = model;
        c.timeout_secs = 180;
        let r = TranslationRequest {
            id: 1,
            text: "vc consegue me mandar isso amanhã? 😊\nESP32-P4, GPIO38, 3,3 V".into(),
            operation: "translate".into(),
        };
        let started = Instant::now();
        let text = HttpProvider::new()
            .unwrap()
            .translate(&c, &r, None)
            .await
            .unwrap();
        assert!(!text.trim().is_empty());
        assert!(text.contains("ESP32-P4"));
        assert!(text.contains("GPIO38"));
        println!(
            "Local Ollama integration finished in {} ms; no content logged",
            started.elapsed().as_millis()
        );
    }
    #[tokio::test]
    async fn provider_contracts() {
        let server = MockServer::start().await;
        let r = TranslationRequest {
            id: 1,
            text: "vc consegue me mandar isso amanhã? 😊\nGPIO38".into(),
            operation: "translate".into(),
        };
        for (id, route, response) in [
            (
                "ollama",
                "/api/chat",
                json!({"done":true,"message":{"content":"hi\n😊"}}),
            ),
            (
                "openai",
                "/chat/completions",
                json!({"choices":[{"finish_reason":"stop","message":{"content":"hi\n😊"}}]}),
            ),
            (
                "anthropic",
                "/v1/messages",
                json!({"stop_reason":"end_turn","content":[{"type":"text","text":"hi\n😊"}]}),
            ),
            (
                "libretranslate",
                "/translate",
                json!({"translatedText":"hi\n😊"}),
            ),
        ] {
            let mut c = Config {
                provider: id.into(),
                ..Config::default()
            };
            c.providers.get_mut(id).unwrap().base_url = server.uri();
            c.providers.get_mut(id).unwrap().model = "test-model".into();
            let mut expected = payload(&c, &r);
            if id == "libretranslate" {
                expected["api_key"] = json!("test-key");
            }
            if id == "ollama" {
                Mock::given(method("GET"))
                    .and(path("/api/tags"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_json(json!({"models":[{"name":"test-model"}]})),
                    )
                    .mount(&server)
                    .await;
            }
            let mut mock = Mock::given(method("POST"))
                .and(path(route))
                .and(body_json(expected));
            if id == "openai" {
                mock = mock.and(header("authorization", "Bearer test-key"));
            }
            if id == "anthropic" {
                mock = mock
                    .and(header("x-api-key", "test-key"))
                    .and(header("anthropic-version", "2023-06-01"));
            }
            mock.respond_with(ResponseTemplate::new(200).set_body_json(response))
                .expect(1)
                .mount(&server)
                .await;
            assert_eq!(
                HttpProvider::new()
                    .unwrap()
                    .translate(&c, &r, Some("test-key"))
                    .await
                    .unwrap(),
                "hi\n😊"
            );
        }
    }
    #[test]
    fn google_contract_and_plaintext_decoding() {
        let c = Config {
            provider: "google".into(),
            source: "auto".into(),
            ..Config::default()
        };
        let r = TranslationRequest {
            id: 1,
            text: "a & b".into(),
            operation: "translate".into(),
        };
        assert_eq!(
            payload(&c, &r),
            json!({"q":"a & b","target":"en","format":"text"})
        );
        assert_eq!(
            parse(
                "google",
                &json!({"data":{"translations":[{"translatedText":"a &amp; b"}]}})
            )
            .unwrap(),
            "a & b"
        );
    }
    #[test]
    fn incomplete_responses_block_confirmation() {
        assert!(parse(
            "openai",
            &json!({"choices":[{"finish_reason":"length","message":{"content":"partial"}}]})
        )
        .is_err());
        assert!(parse(
            "anthropic",
            &json!({"stop_reason":"max_tokens","content":[{"type":"text","text":"partial"}]})
        )
        .is_err());
    }
    #[tokio::test]
    async fn errors_do_not_expose_response_or_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429).set_body_string("secret text private"))
            .mount(&server)
            .await;
        let provider = HttpProvider::new().unwrap();
        let e = provider
            .json(provider.client.get(server.uri()), 5)
            .await
            .unwrap_err();
        assert!(!e.contains("secret"));
        assert!(!e.contains(&server.uri()));
    }
    #[tokio::test]
    async fn timeout_covers_response_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(2))
                    .set_body_json(json!({})),
            )
            .mount(&server)
            .await;
        let provider = HttpProvider::new().unwrap();
        assert!(provider
            .json(provider.client.get(server.uri()), 1)
            .await
            .unwrap_err()
            .contains("timeout"));
    }
}
