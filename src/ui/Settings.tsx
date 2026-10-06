// src/ui/Settings.tsx
import { useState } from 'react';
import { Check, LoaderCircle, PlugZap, ShieldCheck, Trash2 } from 'lucide-react';
import { command, message } from '../bridge';
import { languages, localEndpoint, providerNames, type Config, type ProviderId } from '../types';
import LocalModels from './LocalModels';

export default function Settings({ initial, onSave }: { initial: Config; onSave: (config: Config) => Promise<void> }) {
  const [config, setConfig] = useState(() => structuredClone(initial));
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const [secret, setSecret] = useState('');
  const [persist, setPersist] = useState(false);
  const [items, setItems] = useState<string[]>([]);
  const active = config.providers[config.provider];
  const local = config.provider === 'local';
  const styles = !['local', 'google', 'libretranslate'].includes(config.provider);
  function field<K extends keyof Config>(key: K, value: Config[K]) { setConfig(c => ({ ...c, [key]: value })); setNotice(''); }
  function providerField(key: keyof typeof active, value: string | boolean) { setConfig(c => ({ ...c, providers: { ...c.providers, [c.provider]: { ...c.providers[c.provider], ...(key === 'baseUrl' ? { allowRemote: false, allowInsecure: false } : {}), [key]: value } } })); setNotice(''); }
  async function action(fn: () => Promise<void>) { setBusy(true); setNotice(''); try { await fn(); } catch (e) { setNotice(message(e)); } finally { setBusy(false); } }
  async function save() { await onSave(config); setNotice('Configurações salvas.'); }
  return <section className="settings" aria-label="Configurações">
    <div className="section-heading"><div><span className="eyebrow">DO SEU JEITO</span><h2>Configurações</h2></div><ShieldCheck size={26} /></div>
    <fieldset disabled={busy}>
      <div className="grid-two"><label>Provider<select value={config.provider} onChange={e => { field('provider', e.target.value as ProviderId); setSecret(''); setItems([]); }}>{Object.entries(providerNames).map(([id, name]) => <option key={id} value={id}>{name}</option>)}</select></label>
        {!local && <label>Modelo {styles ? '' : '(NMT do serviço)'}<input list="provider-models" value={active.model} disabled={!styles} placeholder={styles ? 'Escolha ou informe um ID de modelo' : 'Gerenciado pelo serviço'} onChange={e => providerField('model', e.target.value)} /></label>}</div>
      <datalist id="provider-models">{items.map(item => <option key={item} value={item} />)}</datalist>
      {local ? <LocalModels source={config.source} target={config.target} /> : <><label>Endpoint<input type="url" value={active.baseUrl} disabled={config.provider === 'google'} onChange={e => { providerField('baseUrl', e.target.value); setSecret(''); setItems([]); }} /></label>
      <div className={`privacy-note ${localEndpoint(active.baseUrl) ? '' : 'remote'}`}><ShieldCheck size={17} /><span>{localEndpoint(active.baseUrl) ? 'Endpoint no computador. Verifique se o modelo utiliza serviços externos.' : 'Este endpoint envia seu texto para fora do computador.'}</span></div>
      <label className="checkbox"><input type="checkbox" checked={active.allowRemote} onChange={e => providerField('allowRemote', e.target.checked)} />Autorizo envio remoto por este endpoint ou modelo cloud. Desativado por padrão.</label>
      {!localEndpoint(active.baseUrl) && active.baseUrl.startsWith('http:') && <label className="checkbox"><input type="checkbox" checked={active.allowInsecure} onChange={e => providerField('allowInsecure', e.target.checked)} />Autorizo HTTP remoto sem criptografia (texto e credencial ficam expostos).</label>}
      <button className="secondary" onClick={() => void action(async () => { await onSave(config); const result = await command<{ items: string[]; kind: string; elapsedMs: number }>('inspect_provider'); setItems(result.kind === 'models' ? result.items : []); setNotice(`Conectado em ${result.elapsedMs} ms. ${result.kind === 'models' ? 'Modelos' : 'Idiomas'}: ${result.items.join(', ') || 'nenhum disponível'}`); })}><PlugZap size={16} />Salvar e testar conexão / listar</button>
      </>}
      {!local && config.provider !== 'ollama' && <div className="credential-box"><label>Credencial · somente escrita<input type="password" autoComplete="off" value={secret} placeholder="Nunca exibimos a credencial salva" onChange={e => setSecret(e.target.value)} /></label>
        <label className="checkbox"><input type="checkbox" checked={persist} onChange={e => setPersist(e.target.checked)} />Salvar no cofre do sistema. Desmarcado: somente nesta sessão.</label>
        <div className="button-row"><button className="secondary" disabled={!secret.trim()} onClick={() => void action(async () => { await onSave(config); try { await command('set_credential', { secret, persist }); setNotice(persist ? 'Credencial salva no cofre.' : 'Credencial disponível somente na sessão.'); } finally { setSecret(''); } })}><ShieldCheck size={16} />Salvar credencial</button><button className="text-button" onClick={() => void action(async () => { await onSave(config); await command('remove_credential'); setNotice('Credencial removida.'); })}><Trash2 size={15} />Remover</button></div>
        <p className="help">Google usa uma API key para Cloud Translation Basic v2; exige projeto, API ativada e cobrança configurada. OpenAI e Anthropic usam créditos de API separados das assinaturas dos chats.</p>
      </div>}
      <div className="divider" />
      <h3>Idioma e voz</h3><div className="grid-two"><label>Idioma de origem<select value={config.source} onChange={e => field('source', e.target.value)}><option value="auto">Detectar pelo provider</option>{languages.map(([id, label]) => <option value={id} key={id}>{label}</option>)}</select></label><label>Idioma de destino<select value={config.target} onChange={e => field('target', e.target.value)}>{languages.map(([id, label]) => <option value={id} key={id}>{label}</option>)}</select></label></div>
      <div className="grid-two"><label>Tom<select value={config.tone} disabled={!styles} onChange={e => field('tone', e.target.value)}>{[['auto','Automático'],['casual','Casual'],['neutral','Neutro'],['professional','Profissional'],['technical','Técnico'],['literal','Literal']].map(([id, label]) => <option value={id} key={id}>{label}</option>)}</select></label><label>Tratamento<select value={config.formality} disabled={!styles} onChange={e => field('formality', e.target.value)}><option value="auto">Automático</option><option value="informal">Informal</option><option value="formal">Formal</option></select></label></div>
      {styles ? <><label>Relação<select value={config.relationship} onChange={e => field('relationship', e.target.value)}><option value="auto">Automático</option><option value="friend">Amigo</option><option value="colleague">Colega</option><option value="client">Cliente</option></select></label><label>Preferências adicionais para o prompt<textarea rows={2} maxLength={4000} value={config.promptExtra} placeholder="Opcional. Preferências de tradução, sem incluir segredos." onChange={e => field('promptExtra', e.target.value)} /></label></> : <p className="help">Este provider não oferece controle de tom/tratamento nem variantes regionais. São enviados códigos base de idioma; nenhum outro provider é chamado para adaptar o texto.</p>}
      <div className="grid-two"><label>Debounce (300–600 ms)<input type="number" min="300" max="600" value={config.debounceMs} onChange={e => field('debounceMs', Number(e.target.value))} /></label><label>Timeout (5–180 s)<input type="number" min="5" max="180" value={config.timeoutSecs} onChange={e => field('timeoutSecs', Number(e.target.value))} /></label></div>
      {styles && <label>Temperatura (0–1)<input type="number" step="0.1" min="0" max="1" value={config.temperature} onChange={e => field('temperature', Number(e.target.value))} /></label>}
      <div className="divider" /><h3>Desktop</h3><label>Atalho para escrever<input value={config.composeShortcut} onChange={e => field('composeShortcut', e.target.value)} /></label><div className="grid-two"><label>Traduzir seleção<input value={config.selectionShortcut} onChange={e => field('selectionShortcut', e.target.value)} /></label><label>Repetir última tradução<input value={config.repeatShortcut} onChange={e => field('repeatShortcut', e.target.value)} /></label></div>
      <p className="help">Conflitos de registro são informados. Ctrl+Alt pode conflitar com AltGr em alguns layouts.</p>
      <label>Tema<select value={config.theme} onChange={e => field('theme', e.target.value as Config['theme'])}><option value="system">Sistema</option><option value="dark">Escuro</option><option value="light">Claro</option></select></label><label className="checkbox"><input type="checkbox" checked={config.autostart} onChange={e => field('autostart', e.target.checked)} />Iniciar com o sistema</label>
      <div className="privacy-note"><ShieldCheck size={18} /><span>Histórico persistente desativado. Rascunhos e cache vivem somente na memória. Sem telemetria ou fallback remoto automático.</span></div>
      <button className="primary save" onClick={() => void action(save)}>{busy ? <LoaderCircle className="spin" size={17} /> : <Check size={17} />}Salvar configurações</button>
    </fieldset>
    {notice && <p role="status" className="notice">{notice}</p>}
  </section>;
}
