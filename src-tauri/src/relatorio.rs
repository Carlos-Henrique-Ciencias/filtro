use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use serde_json::Value;

pub fn gerar_dashboard_final() -> Result<(), Box<dyn std::error::Error>> {
    let aprovados_path = "../aprovados.json"; // Busca os sobreviventes na raiz
    let relatorio_path = "../dashboard_relatorio.html"; // Salva na raiz

    // 🎯 1. Abre o arquivo de aprovados em modo fluxo (BufReader)
    let arquivo_aprovados = match OpenOptions::new().read(true).open(aprovados_path) {
        Ok(file) => file,
        Err(_) => {
            println!("[📊 ROBO 3] Nenhum arquivo 'aprovados.json' encontrado na raiz.");
            return Ok(());
        }
    };

    // 🎯 2. Cria/Trunca o arquivo HTML final para ir escrevendo por partes
    let mut html_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(relatorio_path)?;

    // 🎯 3. Escreve a primeira parte do cabeçalho HTML (Deixamos o contador para injetar dinamicamente ou fixar no topo)
    let html_cabecalho = "<!DOCTYPE html>
<html lang='pt-BR'>
<head>
    <meta charset='UTF-8'>
    <title>🛡️ Dashboard PNCP Minerador</title>
    <style>
        body { font-family: sans-serif; background: #121214; color: #e1e1e6; margin: 40px; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; background: #202024; border-radius: 8px; overflow: hidden; }
        th { background: #00875f; color: white; text-align: left; padding: 15px; }
    </style>
</head>
<body>
    <h1 style='color:#00b37e;'>🛡️ Relatório do Caçador PNCP</h1>
    <p style='color:#8d8d99;'>Painel de editais de software aprovados no pente fino:</p>
    <table>
        <thead>
            <tr>
                <th>Edital</th>
                <th>Órgão Público</th>
                <th>Ação</th>
            </tr>
        </thead>
        <tbody>\n";

    html_file.write_all(html_cabecalho.as_bytes())?;

    let reader = BufReader::new(arquivo_aprovados);
    let mut total_itens = 0;

    // 🎯 4. Tritura o arquivo de aprovados linha por linha, jogando as linhas da tabela direto no HD
    for linha in reader.lines() {
        if let Ok(linha_texto) = linha {
            if linha_texto.trim().is_empty() { continue; }

            if let Ok(item) = serde_json::from_str::<Value>(&linha_texto) {
                let titulo = item["title"].as_str().unwrap_or("Edital Sem Título");
                let orgao = item["orgao_nome"].as_str().unwrap_or("Órgão Omitido");
                let url = item["item_url"].as_str().unwrap_or("#");

                let tr_html = format!(
                    "<tr>
                        <td style='padding:12px; border-bottom:1px solid #29292e;'><b>{}</b></td>
                        <td style='padding:12px; border-bottom:1px solid #29292e; color:#a8a8b3;'>{}</td>
                        <td style='padding:12px; border-bottom:1px solid #29292e;'>
                            <a href='{}' target='_blank' style='color:#00b37e; text-decoration:none; font-weight:bold;'>Abrir no PNCP ➔</a>
                        </td>
                    </tr>\n",
                    titulo, orgao, url
                );

                // Cospe a linha diretamente no arquivo HTML, liberando a memória RAM na hora
                html_file.write_all(tr_html.as_bytes())?;
                total_itens += 1;
            }
        }
    }

    // Se o loop rodou e não achou nenhum edital aprovado, joga a linha de aviso de lista vazia
    if total_itens == 0 {
        let sem_dados = "<tr><td colspan='3' style='padding:20px; text-align:center; color:#7c7c8a;'>Nenhum edital elegível hoje.</td></tr>\n";
        html_file.write_all(sem_dados.as_bytes())?;
    }

    // 🎯 5. Fecha as tags do arquivo HTML
    let html_rodape = "        </tbody>
    </table>
</body>
</html>";
    
    html_file.write_all(html_rodape.as_bytes())?;

    println!("\n--- 📊 ROBÔ 3: RELATÓRIO COMPILADO COM SUCESSO ---");
    println!("Arquivo 'dashboard_relatorio.html' atualizado na raiz! Total processado: {}", total_itens);

    Ok(())
}