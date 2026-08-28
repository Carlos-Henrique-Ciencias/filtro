use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use regex::Regex;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write, Cursor};
use std::panic;

// 🎯 A FUNÇÃO RECEBE OS VETORES DO FRONTEND
pub async fn executar_escavador(escav_inc: Vec<String>, escav_exc: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(20))
        .build()?;

    let fila_path = "../fila_trabalho.json";
    let aprovados_path = "../aprovados.json";

    // 🎯 REGEX DINÂMICO PARA OS ANEXOS (GERADO PELO FRONTEND)
    let str_inc = if escav_inc.is_empty() { r"(?i).*".to_string() } else { format!(r"(?i)({})", escav_inc.join("|")) };
    let re_inc_arquivos = Regex::new(&str_inc).unwrap();

    let str_exc = if escav_exc.is_empty() { r"PALAVRA_IMPOSSIVEL_DE_EXISTIR_999".to_string() } else { format!(r"(?i)({})", escav_exc.join("|")) };
    let re_exc_arquivos = Regex::new(&str_exc).unwrap();

    let arquivo_fila = match OpenOptions::new().read(true).open(fila_path) {
        Ok(file) => file,
        Err(_) => {
            println!("[🏗️ ROBO 2] Nenhuma fila encontrada ou o arquivo 'fila_trabalho.json' está vazio.");
            return Ok(());
        }
    };

    let mut arquivo_aprovados = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(aprovados_path)?;

    let reader = BufReader::new(arquivo_fila);
    let mut contador_barrados = 0;
    let mut contador_aprovados = 0;

    println!("--- 🏗️ ROBÔ 2: ASSUMINDO ESCAVAÇÃO PESADA (STREAMING DE RAM) ---");
    println!("Traturando fila de editais em background...\n");

    for linha in reader.lines() {
        if let Ok(linha_texto) = linha {
            if linha_texto.trim().is_empty() { continue; }

            if let Ok(item) = serde_json::from_str::<Value>(&linha_texto) {
                let cnpj = item["orgao_cnpj"].as_str().unwrap_or("");
                let ano = item["ano"].as_str().unwrap_or("");
                let seq = item["numero_sequencial"].as_str().unwrap_or("");
                let orgao_nome = item["orgao_nome"].as_str().unwrap_or("N/A");
                let titulo = item["title"].as_str().unwrap_or("");

                println!("[🔍 TRITURANDO ARQUIVOS] Órgão: {}", orgao_nome);

                let texto_anexos = ler_todos_os_formatos_na_ram(&client, cnpj, ano, seq).await;

                let mut item_modificado = item.clone();

                if !texto_anexos.is_empty() {

                    // 🎯 VERIFICAÇÃO DUPLA (INCLUSÃO E EXCLUSÃO)
                    let passou_inc = re_inc_arquivos.is_match(&texto_anexos);
                    let barrado_exc = re_exc_arquivos.is_match(&texto_anexos);

                    if passou_inc && !barrado_exc {
                        println!("   [✨ CONTEÚDO APROVADO] Contém os requisitos e passou limpo nas travas!");
                        item_modificado["alerta_manual"] = serde_json::json!(false);

                        let mut aprovado_str = item_modificado.to_string();
                        aprovado_str.push('\n');
                        let _ = arquivo_aprovados.write_all(aprovado_str.as_bytes());
                        contador_aprovados += 1;
                    } else {
                        println!("   [❌ BARRADO NO PENTE FINO] Não atendeu aos critérios de inclusão/exclusão da tela.");
                        contador_barrados += 1;
                    }
                } else {
                    println!("   [⚠️ ALERTA MANUAL] Arquivos sem texto extraível. Forçando aprovação com ressalva.");
                    item_modificado["alerta_manual"] = serde_json::json!(true);
                    item_modificado["title"] = serde_json::json!(format!("⚠️ [REVER NO OLHO] {}", titulo));

                    let mut aprovado_str = item_modificado.to_string();
                    aprovado_str.push('\n');
                    let _ = arquivo_aprovados.write_all(aprovado_str.as_bytes());
                    contador_aprovados += 1;
                }
                println!("--------------------------------------------------");
            }
        }
    }

    println!("\nTriagem de Arquivos Concluída.");
    println!("Barrados no pente fino dos anexos: {} | Sobreviventes salvos para o relatório: {}", contador_barrados, contador_aprovados);

    Ok(())
}

async fn ler_todos_os_formatos_na_ram(client: &Client, cnpj: &str, ano: &str, seq: &str) -> String {
    let mut texto_acumulado = String::new();

    for num_documento in 1..=10 {
        // 🎯 RESTAURADA A URL ORIGINAL PERFEITA QUE TRAZ O ARQUIVO
        let arquivos_url = format!(
            "https://pncp.gov.br/pncp-api/v1/orgaos/{}/compras/{}/{}/arquivos/{}",
            cnpj, ano, seq, num_documento
        );

        if let Ok(resp) = client.get(&arquivos_url).send().await {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes().await {

                    // ESTRATÉGIA ZIP E DOCX
                    if bytes.starts_with(b"PK\x03\x04") {
                        let reader = Cursor::new(&bytes);
                        if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                            for i in 0..archive.len() {
                                if let Ok(mut file) = archive.by_index(i) {
                                    let nome_interno = file.name().to_lowercase();

                                    if nome_interno.ends_with(".pdf") {
                                        let mut pdf_bytes = Vec::new();
                                        if std::io::copy(&mut file, &mut pdf_bytes).is_ok() {
                                            let pdf_bytes_clone = pdf_bytes.clone();
                                            let texto_extraido = panic::catch_unwind(|| {
                                                pdf_extract::extract_text_from_mem(&pdf_bytes_clone)
                                            });

                                            let texto_pdf = match texto_extraido {
                                                Ok(Ok(t)) => t,
                                                Ok(Err(_)) => String::new(),
                                                Err(_) => {
                                                    println!("   [☢️ PDF TÓXICO] O edital estava corrompido e tentou matar o robô. O escudo segurou!");
                                                    String::new()
                                                }
                                            };
                                            texto_acumulado.push_str(&texto_pdf);
                                            texto_acumulado.push(' ');
                                        }
                                    }
                                    else if nome_interno == "word/document.xml" {
                                        let mut xml_bytes = Vec::new();
                                        if std::io::copy(&mut file, &mut xml_bytes).is_ok() {
                                            if let Ok(xml_texto) = String::from_utf8(xml_bytes) {
                                                let re_tags = Regex::new(r"<[^>]*>").unwrap();
                                                let texto_docx = re_tags.replace_all(&xml_texto, " ");
                                                texto_acumulado.push_str(&texto_docx);
                                                texto_acumulado.push(' ');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } 
                    // ESTRATÉGIA PDF PADRÃO
                    else {
                        let bytes_clone = bytes.clone();
                        let texto_extraido = panic::catch_unwind(|| {
                            pdf_extract::extract_text_from_mem(&bytes_clone)
                        });

                        let texto_pdf = match texto_extraido {
                            Ok(Ok(t)) => t,
                            Ok(Err(_)) => String::new(),
                            Err(_) => {
                                println!("   [☢️ PDF TÓXICO] O edital estava corrompido e tentou matar o robô. O escudo segurou!");
                                String::new()
                            }
                        };
                        texto_acumulado.push_str(&texto_pdf);
                        texto_acumulado.push(' ');
                    }
                }
            } else {
                break;
            }
        }
    }

    texto_acumulado
}