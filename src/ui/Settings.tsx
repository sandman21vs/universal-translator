// src/ui/Settings.tsx
import { useRef, useState } from 'react';
import { LoaderCircle, PlugZap, Trash2 } from 'lucide-react';
import { command, message } from '../bridge';
import { languages, localEndpoint, providerNames, type Config, type ProviderConfig, type ProviderId } from '../types';
import LocalModels from './LocalModels';

const services = (Object.keys(providerNames) as ProviderId[]).filter(id => id !== 'local');
export default function Settings({ initial, onSave }: { initial: Config; onSave: (config: Config) => Promise<void> }) {
  const [config, setConfig] = useState(() => structuredClone(initial));
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const [secret, setSecret] = useState('');
  const [persist, setPersist] = useState(false);
  const [items, setItems] = useState<string[]>([]);
  // Choices save at once; typed fields save when the field is left, so half-typed values are never stored.
  const latest = useRef(config);
  const saves = useRef(Promise.resolve());
  const lastService = useRef<ProviderId>(initial.provider === 'local' ? 'ollama' : initial.provider);
  const active = config.providers[config.provider];
  const local = config.provider === 'local';
  const styles = !['local', 'google', 'libretranslate'].includes(config.provider);
  const remote = !localEndpoint(active.baseUrl);
  function store(next: Config) { saves.current = saves.current.then(() => onSave(next)).then(() => setNotice('Salvo.'), e => setNotice(message(e))); }
  function update(next: Config, commit: boolean) { latest.current = next; setConfig(next); setNotice(''); if (commit) store(next); }
  function field<K extends keyof Config>(key: K, value: Config[K], commit = true) { update({ ...latest.current, [key]: value }, commit); }
  function providerField(key: keyof ProviderConfig, value: string | boolean, commit = true) { const c = latest.current; update({ ...c, providers: { ...c.providers, [c.provider]: { ...c.providers[c.provider], ...(key === 'baseUrl' ? { allowRemote: false, allowInsecure: false } : {}), [key]: value } } }, commit); }
  function engine(provider: ProviderId) { if (provider !== 'local') lastService.current = provider; setSecret(''); setItems([]); field('provider', provider); }
  const commit = () => store(latest.current);
  async function action(fn: () => Promise<void>) { setBusy(true); setNotice(''); try { await fn(); } catch (e) { setNotice(message(e)); } finally { setBusy(false); } }
  return <section className="settings" aria-label="Configurações">
    <h2>Configurações</h2>
    <fieldset disabled={busy}>
      <h3>Idiomas</h3>
      <div className="group"><label>Traduzir de<select value={config.source} onChange={e => field('source', e.target.value)}><option value="auto">Detectar automaticamente</option>{languages.map(([id, label]) => <option value={id} key={id}>{label}</option>)}</select></label><label>Para<select value={config.target} onChange={e => field('target', e.target.value)}>{languages.map(([id, label]) => <option value={id} key={id}>{label}</option>)}</select></label></div>
      <h3>Motor de tradução</h3>
      <div className="segmented wide" role="group" aria-label="Motor de tradução"><button className={local ? 'active' : ''} aria-pressed={local} onClick={() => engine('local')}>No computador</button><button className={local ? '' : 'active'} aria-pressed={!local} onClick={() => engine(lastService.current)}>Usar IA</button></div>
      <p className="help">{local ? 'Privado e sem conta: o texto nunca sai do computador. Basta baixar os idiomas uma vez.' : 'Usa um serviço de IA que você escolhe. Costuma traduzir com mais naturalidade, mas pode exigir conta e enviar o texto para fora do computador.'}</p>
      <datalist id="provider-models">{items.map(item => <option key={item} value={item} />)}</datalist>
      {local ? <LocalModels source={config.source} target={config.target} /> : <>
        <div className="group"><label>Serviço<select value={config.provider} onChange={e => engine(e.target.value as ProviderId)}>{services.map(id => <option key={id} value={id}>{providerNames[id]}</option>)}</select></label>
          {styles && <label>Modelo<input list="provider-models" value={active.model} placeholder="Nome do modelo" onChange={e => providerField('model', e.target.value, false)} onBlur={commit} /></label>}
          {config.provider !== 'ollama' && <><label>Chave de acesso<input type="password" autoComplete="off" value={secret} placeholder="A chave salva nunca é exibida" onChange={e => setSecret(e.target.value)} /></label>
          <label className="checkbox"><input type="checkbox" checked={persist} onChange={e => setPersist(e.target.checked)} />Guardar a chave no cofre do sistema (senão, vale só até fechar o app)</label></>}
          <label className="checkbox"><input type="checkbox" checked={active.allowRemote} onChange={e => providerField('allowRemote', e.target.checked)} />Permitir enviar meu texto para este serviço</label></div>
        <p className="help">{remote ? 'Este serviço recebe seu texto pela internet.' : 'Este serviço roda no seu computador. Verifique se o modelo escolhido não usa a nuvem.'}{config.provider !== 'ollama' ? ' OpenAI e Anthropic cobram créditos de API, separados das assinaturas dos chats; o Google exige um projeto com cobrança ativada.' : ''}</p>
        <div className="button-row">{config.provider !== 'ollama' && <><button className="secondary" disabled={!secret.trim()} onClick={() => void action(async () => { await onSave(config); try { await command('set_credential', { secret, persist }); setNotice(persist ? 'Chave salva no cofre.' : 'Chave disponível somente nesta sessão.'); } finally { setSecret(''); } })}>Salvar chave</button><button className="text-button" onClick={() => void action(async () => { await onSave(config); await command('remove_credential'); setNotice('Chave removida.'); })}><Trash2 size={13} />Remover chave</button></>}
          <button className="secondary" onClick={() => void action(async () => { await onSave(config); const result = await command<{ items: string[]; kind: string; elapsedMs: number }>('inspect_provider'); setItems(result.kind === 'models' ? result.items : []); setNotice(`Conectado em ${result.elapsedMs} ms. ${result.kind === 'models' ? 'Modelos' : 'Idiomas'}: ${result.items.join(', ') || 'nenhum disponível'}`); })}>{busy ? <LoaderCircle className="spin" size={14} /> : <PlugZap size={14} />}Testar conexão</button></div>
        {styles ? <><h3>Estilo da tradução</h3>
        <div className="group"><label>Tom<select value={config.tone} onChange={e => field('tone', e.target.value)}>{[['auto','Automático'],['casual','Casual'],['neutral','Neutro'],['professional','Profissional'],['technical','Técnico'],['literal','Literal']].map(([id, label]) => <option value={id} key={id}>{label}</option>)}</select></label><label>Tratamento<select value={config.formality} onChange={e => field('formality', e.target.value)}><option value="auto">Automático</option><option value="informal">Informal</option><option value="formal">Formal</option></select></label>
          <label>Falando com<select value={config.relationship} onChange={e => field('relationship', e.target.value)}><option value="auto">Automático</option><option value="friend">Amigo</option><option value="colleague">Colega</option><option value="client">Cliente</option></select></label>
          <label className="stack">Instruções extras<textarea rows={2} maxLength={4000} value={config.promptExtra} placeholder="Opcional. Ex.: use sempre você, nunca tu." onChange={e => field('promptExtra', e.target.value, false)} onBlur={commit} /></label></div></> : <p className="help">Este serviço traduz sem ajuste de tom ou tratamento e ignora variantes regionais do idioma.</p>}
      </>}
      <h3>Geral</h3>
      <div className="group"><label>Atalho da janelinha<input value={config.composeShortcut} onChange={e => field('composeShortcut', e.target.value, false)} onBlur={commit} /></label>
        <label>Aparência<select value={config.theme} onChange={e => field('theme', e.target.value as Config['theme'])}><option value="system">Igual ao sistema</option><option value="dark">Escura</option><option value="light">Clara</option></select></label><label className="checkbox"><input type="checkbox" checked={config.autostart} onChange={e => field('autostart', e.target.checked)} />Abrir ao iniciar o computador</label></div>
      <details className="advanced"><summary>Avançado</summary>
        <div className="group"><label>Espera antes de traduzir (300–600 ms)<input type="number" min="300" max="600" value={config.debounceMs} onChange={e => field('debounceMs', Number(e.target.value), false)} onBlur={commit} /></label><label>Tempo limite (5–180 s)<input type="number" min="5" max="180" value={config.timeoutSecs} onChange={e => field('timeoutSecs', Number(e.target.value), false)} onBlur={commit} /></label>
          {styles && <label>Temperatura (0–1)<input type="number" step="0.1" min="0" max="1" value={config.temperature} onChange={e => field('temperature', Number(e.target.value), false)} onBlur={commit} /></label>}
          {!local && <label>Endereço do serviço<input type="url" value={active.baseUrl} disabled={config.provider === 'google'} onChange={e => { setSecret(''); setItems([]); providerField('baseUrl', e.target.value, false); }} onBlur={commit} /></label>}
          {!local && remote && active.baseUrl.startsWith('http:') && <label className="checkbox"><input type="checkbox" checked={active.allowInsecure} onChange={e => providerField('allowInsecure', e.target.checked)} />Permitir conexão sem criptografia (texto e chave ficam expostos)</label>}
          <label>Atalho para traduzir a seleção<input value={config.selectionShortcut} onChange={e => field('selectionShortcut', e.target.value, false)} onBlur={commit} /></label><label>Atalho para repetir a última tradução<input value={config.repeatShortcut} onChange={e => field('repeatShortcut', e.target.value, false)} onBlur={commit} /></label></div>
        <p className="help">Atalhos em conflito são avisados; Ctrl+Alt pode conflitar com AltGr em alguns teclados. Nada é guardado em histórico: rascunhos e cache vivem só na memória, sem telemetria.</p>
      </details>
    </fieldset>
    <p role="status" className="notice">{notice}</p>
  </section>;
}
