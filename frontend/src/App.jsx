import { createSignal, For, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// 🎯 COMPONENTE DE TAGS
function TagInput(props) {
    const [inputVal, setInputVal] = createSignal("");

    const adicionar = () => {
        const val = inputVal().trim();
        if (val && !props.tags().includes(val)) {
            props.setTags([...props.tags(), val]);
            setInputVal("");
        }
    };

    const remover = (index) => {
        const novasTags = [...props.tags()];
        novasTags.splice(index, 1);
        props.setTags(novasTags);
    };

    return (
        <div style="margin-bottom: 15px;">
            <label style="display: block; margin-bottom: 5px; font-weight: bold; font-size: 13px; color: #c4c4cc;">{props.label}</label>
            <div style="display: flex; gap: 8px;">
                <input
                    type="text" value={inputVal()}
                    onInput={(e) => setInputVal(e.currentTarget.value)}
                    onKeyDown={(e) => { if (e.key === "Enter") adicionar(); }}
                    placeholder={props.placeholder}
                    style={`flex: 1; padding: 10px; background: #121214; border: 1px solid ${props.cor}; color: white; border-radius: 4px; outline: none; font-size: 14px;`}
                />
                <button type="button" onClick={adicionar} style={`padding: 0 15px; background: ${props.cor}; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: bold;`}>Add</button>
            </div>
            <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px;">
                <For each={props.tags()}>
                    {(tag, i) => (
                        <span style={`background: #29292e; color: #e1e1e6; padding: 4px 8px; border-radius: 4px; font-size: 12px; display: flex; align-items: center; gap: 6px; border-left: 2px solid ${props.cor};`}>
                            {tag}
                            <button type="button" onClick={() => remover(i())} style="background: none; border: none; color: #f75a68; cursor: pointer; font-weight: bold; font-size: 12px; padding: 0;">✕</button>
                        </span>
                    )}
                </For>
            </div>
        </div>
    );
}

function App() {
    // VARIÁVEIS DA API
    const [palavraChave, setPalavraChave] = createSignal("");
    const [uf, setUf] = createSignal("TODOS");
    const [dataLimite, setDataLimite] = createSignal("2026-05-28");

    // 🎯 VARIÁVEIS DOS FILTROS (TAGS) - Agora começam vazias para o cliente!
    const [mainInc, setMainInc] = createSignal([]);
    const [mainExc, setMainExc] = createSignal([]);
    const [escavInc, setEscavInc] = createSignal([]);
    const [escavExc, setEscavExc] = createSignal([]);

    const [status, setStatus] = createSignal("Pronto para iniciar");
    const [carregando, setCarregando] = createSignal(false);
    const [linksAprovados, setLinksAprovados] = createSignal([]);

    const carregarHistoricoDoPC = async () => {
        try {
            const respostaHistorico = await invoke("puxar_historico_salvo");
            const dadosReais = JSON.parse(respostaHistorico);
            if (Array.isArray(dadosReais) && dadosReais.length > 0) {
                const formatados = dadosReais.map((item, index) => ({
                    id_unico: item.id || `edital-${index}-${Date.now()}`,
                    titulo: item.title || "Edital Salvo",
                    orgao: item.orgao_nome || "Órgão Não Informado",
                    url: item.item_url || "https://pncp.gov.br"
                }));
                setLinksAprovados(formatados);
                setStatus(`📂 Histórico recuperado! ${formatados.length} editais carregados do PC.`);
            }
        } catch (e) {
            console.error("Erro ao puxar histórico do HD:", e);
        }
    };

    onMount(() => { carregarHistoricoDoPC(); });

    const iniciarBusca = async () => {
        setCarregando(true);
        setStatus("🏃‍♂️ Robôs operando na RAM... Aguarde.");
        setLinksAprovados([]);

        try {
            const respostaRust = await invoke("disparar_esteira_busca", {
                palavraChave: palavraChave(),
                uf: uf(),
                dataLimite: dataLimite(),
                mainInc: mainInc(),
                mainExc: mainExc(),
                escavInc: escavInc(),
                escavExc: escavExc()
            });
            const dadosReais = JSON.parse(respostaRust);

            if (Array.isArray(dadosReais) && dadosReais.length > 0) {
                const formatados = dadosReais.map((item, index) => ({
                    id_unico: item.id || `edital-${index}-${Date.now()}`,
                    titulo: item.title || "Edital Aprovado",
                    orgao: item.orgao_nome || "Órgão Não Informado",
                    url: item.item_url || "https://pncp.gov.br"
                }));
                setLinksAprovados(formatados);
                setStatus(`✅ Triagem concluída! ${formatados.length} editais carregados na tela.`);
            } else {
                setStatus("✅ Esteira concluída. Nenhum edital sobrevivente localizado.");
            }
        } catch (erro) {
            setStatus(`❌ Erro no processamento: ${erro}`);
        } finally {
            setCarregando(false);
        }
    };

    const abrirNoNavegador = async (urlAlvo) => {
        if (!urlAlvo) return;
        try {
            await invoke("abrir_no_navegador_rust", { url: String(urlAlvo).trim() });
        } catch (err) {
            console.error("Erro ao mandar URL para o Rust:", err);
        }
    };

    return (
        <div style="min-height: 100vh; background: #121214; color: #e1e1e6; font-family: sans-serif; display: flex; align-items: center; justify-content: center; padding: 20px; box-sizing: border-box;">
            <div style="width: 100%; max-width: 850px; background: #202024; padding: 30px; border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.4); box-sizing: border-box;">

                <h2 style="color: #00b37e; margin-top: 0; display: flex; align-items: center; gap: 8px; font-size: 26px;">🛡️ Centro de Comando</h2>
                <p style="color: #8d8d99; font-size: 14px; margin-bottom: 30px;">Controle da Esteira PNCP em Tempo Real</p>

                {/* FILTROS DA API */}
                <div style="margin-bottom: 25px; display: flex; gap: 15px;">
                    <div style="flex: 2;">
                        <label style="display: block; margin-bottom: 10px; font-weight: bold; font-size: 14px; color: #c4c4cc;">Palavra-chave (API):</label>
                        <input
                            type="text" value={palavraChave()} onInput={(e) => setPalavraChave(e.currentTarget.value)}
                            placeholder="Ex: software, cameras de segurança"
                            style="width: 100%; padding: 14px; background: #121214; border: 1px solid #29292e; color: white; border-radius: 4px; box-sizing: border-box; font-size: 15px; outline: none;"
                        />
                    </div>
                    <div style="flex: 1;">
                        <label style="display: block; margin-bottom: 10px; font-weight: bold; font-size: 14px; color: #c4c4cc;">Estado (UF):</label>
                        <select
                            value={uf()} onChange={(e) => setUf(e.currentTarget.value)}
                            style="width: 100%; padding: 14px; background: #121214; border: 1px solid #29292e; color: white; border-radius: 4px; box-sizing: border-box; font-size: 15px; outline: none; cursor: pointer;"
                        >
                            <option value="TODOS">🌍 Todos</option>
                            <option value="AC">AC</option><option value="AL">AL</option><option value="AP">AP</option><option value="AM">AM</option><option value="BA">BA</option><option value="CE">CE</option><option value="DF">DF</option><option value="ES">ES</option><option value="GO">GO</option><option value="MA">MA</option><option value="MT">MT</option><option value="MS">MS</option><option value="MG">MG</option><option value="PA">PA</option><option value="PB">PB</option><option value="PR">PR</option><option value="PE">PE</option><option value="PI">PI</option><option value="RJ">RJ</option><option value="RN">RN</option><option value="RS">RS</option><option value="RO">RO</option><option value="RR">RR</option><option value="SC">SC</option><option value="SP">SP</option><option value="SE">SE</option><option value="TO">TO</option>
                        </select>
                    </div>
                    <div style="flex: 1;">
                        <label style="display: block; margin-bottom: 10px; font-weight: bold; font-size: 14px; color: #c4c4cc;">Data Limite:</label>
                        <input
                            type="text" value={dataLimite()} onInput={(e) => setDataLimite(e.currentTarget.value)}
                            placeholder="AAAA-MM-DD"
                            style="width: 100%; padding: 14px; background: #121214; border: 1px solid #29292e; color: white; border-radius: 4px; box-sizing: border-box; font-size: 15px; outline: none; font-family: monospace;"
                        />
                    </div>
                </div>

                {/* PAINEL DE TAGS */}
                <div style="display: flex; gap: 20px; margin-bottom: 30px;">
                    <div style="flex: 1; background: #1a1a1e; padding: 15px; border-radius: 6px; border: 1px solid #29292e;">
                        <h3 style="margin-top: 0; font-size: 15px; color: #00b37e;">🔎 Filtros Descrição (EXIBIDO)</h3>
                        <TagInput label="Obrigatório ter:" tags={mainInc} setTags={setMainInc} placeholder="Ex: tecnologia" cor="#00875f" />
                        <TagInput label="NÃO pode ter:" tags={mainExc} setTags={setMainExc} placeholder="Ex: limpeza" cor="#f75a68" />
                    </div>

                    <div style="flex: 1; background: #1a1a1e; padding: 15px; border-radius: 6px; border: 1px solid #29292e;">
                        <h3 style="margin-top: 0; font-size: 15px; color: #00b37e;">📄 Filtros Documentos (EDITAL)</h3>
                        <TagInput label="Obrigatório ter:" tags={escavInc} setTags={setEscavInc} placeholder="Vazio = Aceita tudo" cor="#00875f" />
                        <TagInput label="NÃO pode ter:" tags={escavExc} setTags={setEscavExc} placeholder="Ex: atestado" cor="#f75a68" />
                    </div>
                </div>

                {/* GATILHO E STATUS */}
                <button onClick={iniciarBusca} disabled={carregando()} style={`width: 100%; padding: 16px; color: white; border: none; border-radius: 4px; font-weight: bold; font-size: 16px; cursor: pointer; transition: background 0.2s; ${carregando() ? 'background: #29292e; cursor: not-allowed;' : 'background: #00875f;'}`}>
                    {carregando() ? "🤖 Triturando PDFs na RAM..." : "🔥 Disparar Caçador"}
                </button>

                <div style="margin-top: 30px; padding: 14px; background: #121214; border-radius: 4px; font-size: 14px; border-left: 4px solid #00b37e; margin-bottom: 25px;">
                    <span style="color: #c4c4cc;"><b>Status:</b> </span>
                    <span style="color: #e1e1e6;">{status()}</span>
                </div>

                {/* LISTA DE EXIBIÇÃO */}
                <div style="background: #121214; padding: 16px; border-radius: 4px; border: 1px solid #29292e;">
                    <p style="margin: 0 0 15px 0; font-size: 12px; color: #7c7c8a; font-weight: bold; letter-spacing: 0.5px;">🎯 SOBREVIVENTES EXIBIDOS NA INTERFACE:</p>
                    <div style="display: flex; flex-direction: column; gap: 12px;">
                        {linksAprovados().length === 0 ? (
                            <span style="font-size: 13px; color: #7c7c8a; font-style: italic;">Nenhum edital na lista. Defina os filtros e dispare.</span>
                        ) : (
                            <For each={linksAprovados()}>
                                {(link) => {
                                    const urlEstatica = link.url;
                                    return (
                                        <div role="button" onClick={() => abrirNoNavegador(urlEstatica)} style="text-align: left; background: #202024; padding: 15px; border-radius: 4px; border: 1px solid #29292e; cursor: pointer; width: 100%; display: block; transition: all 0.2s;" onMouseEnter={(e) => { e.currentTarget.style.borderColor = "#00b37e"; e.currentTarget.style.background = "#242429"; }} onMouseLeave={(e) => { e.currentTarget.style.borderColor = "#29292e"; e.currentTarget.style.background = "#202024"; }}>
                                            <div style="font-weight: bold; color: #00b37e; font-size: 15px; margin-bottom: 4px;">➔ {link.titulo}</div>
                                            <div style="font-size: 13px; color: #c4c4cc; margin-bottom: 6px;">{link.orgao}</div>
                                            <div style="font-size: 11px; color: #00875f; word-break: break-all; font-family: monospace; text-decoration: underline;">{urlEstatica}</div>
                                        </div>
                                    );
                                }}
                            </For>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

export default App;