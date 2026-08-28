mod escavador;
mod relatorio;

use tauri::Emitter;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

#[tauri::command]
async fn disparar_esteira_busca(
    app_handle: tauri::AppHandle,
    palavra_chave: String,
    uf: String,
    data_limite: String,
    main_inc: Vec<String>,
    main_exc: Vec<String>,
    escav_inc: Vec<String>,
    escav_exc: Vec<String>
) -> Result<String, String> {
    println!("📡 [INTERFÁCIL] Sinal recebido! Termo API: '{}' | UF: '{}' | Data: {}", palavra_chave, uf, data_limite);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;

    let fila_path = "../fila_trabalho.json";

    // 🎯 REGEX DINÂMICO PARA O MAIN (GERADO PELO FRONTEND)
    let str_inc = if main_inc.is_empty() { r"(?i).*".to_string() } else { format!(r"(?i)\b({})\b", main_inc.join("|")) };
    let re_inc_api = Regex::new(&str_inc).unwrap();

    let str_exc = if main_exc.is_empty() { r"PALAVRA_IMPOSSIVEL_DE_EXISTIR_999".to_string() } else { format!(r"(?i)({})", main_exc.join("|")) };
    let re_exc_api = Regex::new(&str_exc).unwrap();

    let mut arquivo_fila = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(fila_path)
        .map_err(|e| e.to_string())?;

    let mut contador_lidos = 0;
    let mut contador_adicionados = 0;
    let alcancou_fim_real = false;

    println!("\n--- 🏃‍♂️ ROBÔ 1: ACIONADO VIA INTERFACE ---");

    for pagina in 1..=2000 {
        if alcancou_fim_real { break; }

        let q_encoded = palavra_chave.trim().replace(" ", "%20");
        let uf_param = if !uf.is_empty() && uf.to_uppercase() != "TODOS" {
            format!("&ufs={}", uf.to_uppercase())
        } else {
            String::new()
        };

        // URL Geralzão dinâmica usando o seu padrão
        let url = format!(
            "https://pncp.gov.br/api/search/?q={}&tipos_documento=edital&ordenacao=-data&pagina={}&tam_pagina=10&status=recebendo_proposta{}",
            q_encoded, pagina, uf_param
        );

        if let Ok(response) = client.get(&url).send().await {
            if let Ok(json) = response.json::<Value>().await {
                if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
                    if items.is_empty() { break; }

                    for item in items {
                        contador_lidos += 1;

                        let data_verificacao = item["data_atualizacao_pncp"].as_str()
                            .or_else(|| item["createdAt"].as_str())
                            .or_else(|| item["data_inicio_vigencia"].as_str())
                            .unwrap_or("");

                        let data_limpa = if data_verificacao.len() >= 10 { &data_verificacao[0..10] } else { "" };

                        println!("DEBUG: Edital: {} | Data Lida: {} | Limite: {} | É menor? {}",
                                 item["title"].as_str().unwrap_or("Sem Titulo"),
                                 data_limpa,
                                 data_limite.as_str(),
                                 data_limpa < data_limite.as_str());

                        if !data_limpa.is_empty() && data_limpa < data_limite.as_str() {
                            continue;
                        }

                        let id = item["id"].as_str().unwrap_or("").to_string();
                        let titulo = item["title"].as_str().unwrap_or("").to_string();
                        let titulo_min = titulo.to_lowercase();
                        let desc_en = item["description"].as_str().unwrap_or("").to_lowercase();
                        let desc_pt = item["descricao"].as_str().unwrap_or("").to_lowercase();
                        let complementar = item["informacaoComplementar"].as_str().unwrap_or("").to_lowercase();
                        let texto_meta = format!("{} {} {} {}", titulo_min, desc_en, desc_pt, complementar);
                        let orgao = item["orgao_nome"].as_str().unwrap_or("Órgão Omitido");

                        if re_inc_api.is_match(&texto_meta) && !re_exc_api.is_match(&texto_meta) {
                            let cnpj = item["orgao_cnpj"].as_str().unwrap_or("");
                            
                            // 🎯 AQUI: Proteção para garantir a extração do Ano e da Sequência
                            let ano = if let Some(n) = item["ano"].as_u64() {
                                n.to_string()
                            } else {
                                item["ano"].as_str().unwrap_or("2026").to_string()
                            };
                            
                            let seq = if let Some(n) = item["numero_sequencial"].as_u64() {
                                n.to_string()
                            } else {
                                item["numero_sequencial"].as_str().unwrap_or("0").to_string()
                            };

                            let path = item["item_url"].as_str().unwrap_or("");
                            let full_url = if path.contains("/compras/") {
                                format!("https://pncp.gov.br{}", path.replace("/compras/", "/app/editais/"))
                            } else {
                                format!("https://pncp.gov.br/app/editais/{}/{}/{}", cnpj, ano, seq)
                            };

                            let pacote_esteira = serde_json::json!({
                                "id": id, "orgao_cnpj": cnpj, "ano": ano, "numero_sequencial": seq,
                                "orgao_nome": orgao, "title": titulo, "item_url": full_url
                            });

                            let _ = app_handle.emit("edital-encontrado", &pacote_esteira);
                            let mut linha_json = pacote_esteira.to_string();
                            linha_json.push('\n');
                            let _ = arquivo_fila.write_all(linha_json.as_bytes());
                            contador_adicionados += 1;
                        }
                    }
                }
            }
        }
    }

    println!("\n==================================================");
    println!("🏁 [BALANÇO FINAL DO ROBÔ 1 - MINERADOR]");
    println!("📖 Total de editais lidos da API PNCP: {}", contador_lidos);
    println!("📥 Total aprovado no Regex (Fila):      {}", contador_adicionados);
    println!("⏳ Total ignorado por ruído/marcas:     {}", contador_lidos - contador_adicionados);
    println!("==================================================\n");

    let _ = escavador::executar_escavador(escav_inc, escav_exc).await;
    let _ = relatorio::gerar_dashboard_final();

    match std::fs::read_to_string("../aprovados.json") {
        Ok(conteudo) => {
            let mut linhas_validas = Vec::new();
            for linha in conteudo.lines() {
                if !linha.trim().is_empty() {
                    linhas_validas.push(linha.trim());
                }
            }
            let json_final = format!("[{}]", linhas_validas.join(","));
            Ok(json_final)
        }
        Err(_) => Ok("[]".to_string()),
    }
}

#[tauri::command]
fn puxar_historico_salvo() -> Result<String, String> {
    match std::fs::read_to_string("../aprovados.json") {
        Ok(conteudo) => {
            let mut json_array = String::from("[");
            let mut linhas = conteudo.lines().peekable();
            while let Some(linha) = linhas.next() {
                if !linha.trim().is_empty() {
                    json_array.push_str(linha);
                    if linhas.peek().is_some() {
                        json_array.push(',');
                    }
                }
            }
            json_array.push(']');
            Ok(json_array)
        }
        Err(_) => Ok("[]".to_string()),
    }
}

#[tauri::command]
fn abrir_no_navegador_rust(url: String) {
    if !url.is_empty() {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

fn main() {
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            disparar_esteira_busca,
            puxar_historico_salvo,
            abrir_no_navegador_rust
        ])
        .run(tauri::generate_context!())
        .expect("erro ao rodar a aplicação tauri");
}