// src/ui/App.tsx
import { useEffect, useRef, useState, useSyncExternalStore } from 'react';
import { listen } from '@tauri-apps/api/event';
import { isTauri } from '@tauri-apps/api/core';
import { ArrowDown, ArrowRight, Check, Clipboard, CornerDownLeft, Languages, LoaderCircle, LockKeyhole, RotateCcw, Settings2, ShieldCheck, Sparkles, Trash2, X } from 'lucide-react';
import { command, message } from '../bridge';
import { TranslationSession } from '../session';
import { ptBR } from '../i18n';
import { languages, localEndpoint, providerNames, type Bootstrap, type Config, type TranslationResult } from '../types';
import Settings from './Settings';
import CompactOverlay from './CompactOverlay';
import meta from '../../app.meta.json';

const session = new TranslationSession({ translate: (id, text) => command<TranslationResult>('translate', { request: { id, text, operation: 'translate' } }), cancel: id => command('cancel_translation', { id }), insert: (id, text) => command('insert_translation', { id, text }) });
export default function App() {
  const [boot, setBoot] = useState<Bootstrap | null>(null);
  const [page, setPage] = useState<'compose' | 'settings'>('compose');
  const [compact, setCompact] = useState(false);
  const [desktopError, setDesktopError] = useState('');
  const [mode, setMode] = useState<'compose' | 'selection'>('compose');
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);
  const editor = useRef<HTMLTextAreaElement>(null);
  const state = useSyncExternalStore(session.subscribe, session.snapshot);
  async function refresh() { try { const result = await command<Bootstrap>('bootstrap'); setBoot(result); setCompact(result.compact); document.documentElement.dataset.theme = result.config.theme; return result; } catch (e) { setDesktopError(message(e)); return null; } }
  useEffect(() => { document.documentElement.dataset.layout = compact ? 'compact' : 'full'; return () => { delete document.documentElement.dataset.layout; }; }, [compact]);
  useEffect(() => { if (compact && boot) editor.current?.focus(); }, [compact, boot]);
  useEffect(() => {
    let alive = true;
    void command<Bootstrap>('bootstrap').then(result => { if (alive) { setBoot(result); setCompact(result.compact); session.configure(result.config.debounceMs); document.documentElement.dataset.theme = result.config.theme; } }).catch(e => { if (alive) setDesktopError(message(e)); });
    const handlers: Array<[string, (payload: unknown) => void]> = [
      ['compose', () => { setCompact(true); setPage('compose'); setMode('compose'); setDesktopError(''); void refresh(); session.resume(); setTimeout(() => editor.current?.focus(), 0); }],
      ['settings', () => { setCompact(false); setPage('settings'); session.cancel(); void refresh(); }],
      ['selection-open', () => { setCompact(true); setPage('compose'); setMode('selection'); session.cancel(); void refresh(); }],
      ['selection', payload => { setMode('selection'); setDesktopError(''); session.updateText(String(payload)); void refresh(); }],
      ['desktop-error', payload => setDesktopError(String(payload))],
      ['repeat', () => { setCompact(true); setPage('compose'); setMode('selection'); session.repeat(); void refresh(); }],
      ['shortcuts-changed', () => { void refresh(); }],
      ['clear-session', () => { session.clear(); setDesktopError('Sessão limpa.'); void refresh(); }],
    ];
    const registrations = isTauri() ? handlers.map(([event, handler]) => listen(event, e => handler(e.payload))) : [];
    return () => { alive = false; registrations.forEach(p => { void p.then(unlisten => unlisten()); }); };
  }, []);
  async function save(config: Config) { session.cancel(); setSaving(true); try { await command('save_settings', { config }); session.configure(config.debounceMs); await refresh(); } finally { setSaving(false); } }
  async function copy(text: string) { try { await command('copy_text', { text }); setCopied(true); setTimeout(() => setCopied(false), 1500); } catch (e) { setDesktopError(message(e)); } }
  async function cancel() { session.cancel(); try { await command('hide_overlay'); } catch (e) { setDesktopError(message(e)); } }
  async function insert() { if (mode === 'compose' && boot?.insertionAvailable && !saving) await session.insert(); }
  async function openSettings() { session.cancel(); try { await command('open_settings'); setCompact(false); setPage('settings'); await refresh(); } catch (e) { setDesktopError(message(e)); } }
  const c = boot?.config;
  const p = c?.providers[c.provider];
  const label = (id: string) => languages.find(([value]) => value === id)?.[1] || (id === 'auto' ? 'Detecção automática' : id);
  const working = state.phase === 'translating' || state.phase === 'inserting';
  const canInsert = session.canInsert() && mode === 'compose' && !!boot?.insertionAvailable && !saving;
  if (compact) return <CompactOverlay state={state} config={c} editor={editor} selection={mode === 'selection'} available={!!boot?.insertionAvailable} canInsert={canInsert} copied={copied} notice={desktopError || boot?.warning || boot?.shortcutError || ''} onText={text => { setCopied(false); session.updateText(text); }} onOutput={value => session.editOutput(value)} onClose={() => void cancel()} onSettings={() => void openSettings()} onCopy={() => void copy(state.output)} onInsert={() => void insert()} onRetry={() => void session.run()} />;
  return <main className="app" onKeyDown={e => { if (e.key === 'Escape' && state.phase !== 'inserting') { e.preventDefault(); void cancel(); } }}>
    <header className="app-header"><div className="brand"><div className="brand-icon"><Languages size={23} /></div><div><h1>{meta.name}</h1><span>Suas palavras. Em outro idioma.</span></div></div><button className="icon-button" title="Fechar para a bandeja · Esc" aria-label="Fechar para a bandeja" disabled={state.phase === 'inserting'} onClick={() => void cancel()}><X size={20} /></button></header>
    <nav className="tabs" aria-label="Navegação"><button className={page === 'compose' ? 'active' : ''} disabled={saving || state.phase === 'inserting'} onClick={() => { setPage('compose'); setMode('compose'); session.resume(); }}><Sparkles size={16} />Traduzir</button><button className={page === 'settings' ? 'active' : ''} disabled={!boot || saving || state.phase === 'inserting'} onClick={() => { session.cancel(); setPage('settings'); }}><Settings2 size={16} />Configurações</button><span className="version">v{meta.version}</span></nav>
    {(desktopError || boot?.warning || boot?.shortcutError) && <div role="alert" className="alert">{desktopError || boot?.warning || boot?.shortcutError}</div>}
    {page === 'settings' && c ? <Settings initial={c} onSave={save} /> : <section className="compose" aria-label="Compor tradução">
      <div className="intro"><span className="eyebrow">{mode === 'selection' ? 'TRADUZIR SELEÇÃO · LEITURA' : 'ESCREVA COM NATURALIDADE'}</span><h2>Uma conversa, sem barreiras.</h2><p>Escreva como você fala. Revise a tradução antes de usar.</p></div>
      {c?.provider === 'local' && <div className="privacy-note"><ShieldCheck size={17} /><span>Tradução no computador. Instale os idiomas antes do primeiro uso.{c.source !== 'auto' && c.source.split('-')[0] !== 'en' && c.target.split('-')[0] !== 'en' && c.source.split('-')[0] !== c.target.split('-')[0] ? ' Este par usa inglês como idioma intermediário.' : ''}</span><button className="text-button" onClick={() => { session.cancel(); setPage('settings'); }}>Gerenciar idiomas</button></div>}
      <div className="language-route"><div><span className="route-label">DE</span><strong>{c ? label(c.source) : 'Português · Brasil'}</strong></div><ArrowRight size={19} /><div><span className="route-label">PARA</span><strong>{c ? label(c.target) : 'English · US'}</strong></div><button className="icon-button" aria-label="Alterar idiomas" title="Alterar idiomas nas configurações" disabled={!boot} onClick={() => { session.cancel(); setPage('settings'); }}><Settings2 size={16} /></button></div>
      <div className="editor-card"><div className="editor-heading"><label htmlFor="source">{mode === 'selection' ? 'Texto selecionado' : 'Seu texto'}</label><span>{[...state.text].length} / 8.000</span></div><textarea id="source" ref={editor} maxLength={16000} readOnly={mode === 'selection'} disabled={!boot || saving || state.phase === 'inserting'} placeholder="vc consegue me mandar isso amanhã?" value={state.text} onChange={e => { setCopied(false); const text = [...e.target.value].slice(0,8000).join(''); session.updateText(text); }} onKeyDown={e => { if (e.nativeEvent.isComposing) return; if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); if (canInsert) void insert(); } }} /><div className="editor-footer"><span><CornerDownLeft size={13} />Shift + Enter para nova linha</span><button className="text-button" disabled={!state.text || state.phase === 'inserting'} onClick={() => session.updateText('')}><Trash2 size={13} />Limpar</button></div></div>
      <div className="translation-divider"><ArrowDown size={16} /><span>O sentido continua seu.</span></div>
      <div className={`editor-card result-card ${state.phase === 'ready' ? 'ready' : ''}`}><div className="editor-heading"><label htmlFor="translation">Tradução <span className="subtle">· revisável</span></label><span className={`status ${working ? 'working' : ''}`} role="status">{working ? <LoaderCircle size={13} className="spin" /> : state.phase === 'ready' ? <Check size={13} /> : null}{ptBR[state.phase]}</span></div><textarea id="translation" placeholder="Sua tradução aparece aqui…" value={state.output} disabled={state.phase !== 'ready'} onChange={e => session.editOutput(e.target.value)} /><div className="editor-footer"><span>{state.result ? `${state.result.provider} · ${state.result.elapsedMs} ms${state.result.cached ? ' · cache' : ''}` : 'Sem envio automático de mensagens'}</span><button className="text-button" disabled={!boot || !state.text.trim() || saving || state.phase === 'inserting'} onClick={() => void session.run()}><RotateCcw size={13} />Traduzir novamente</button></div></div>
      {state.error && <div role="alert" className="alert">{state.error} Seu texto foi preservado. Você pode copiar o resultado disponível ou traduzir novamente.</div>}
      <div className="action-bar"><button className="secondary" disabled={!state.output || saving || state.phase === 'inserting'} onClick={() => void copy(state.output)}>{copied ? <Check size={17} /> : <Clipboard size={17} />}{copied ? 'Copiado!' : 'Copiar tradução'}</button>{mode === 'selection' && <button className="text-button" disabled={!state.text} onClick={() => void copy(state.text)}>Copiar original</button>}<button className="primary" disabled={!canInsert} title={boot?.insertionAvailable ? 'Insere texto sem enviar a mensagem' : 'Abra com o atalho sobre o campo de destino no Windows'} onClick={() => void insert()}>{state.phase === 'inserting' ? <LoaderCircle className="spin" size={17} /> : <CornerDownLeft size={17} />}Inserir<kbd>Enter</kbd></button></div>
      <div className="destination-note"><LockKeyhole size={14} /><span>{mode === 'selection' ? 'Modo de leitura. Substituição de seleção ainda não disponível.' : boot?.insertionAvailable ? 'Destino registrado. O foco será verificado antes da inserção.' : `Para inserir, abra pelo atalho ${c?.composeShortcut || 'Ctrl+Alt+T'} sobre o campo de destino no Windows.`}</span></div>
    </section>}
    <footer className="app-footer"><span className="provider-status"><span className="dot" />{c ? providerNames[c.provider] : 'Backend desktop necessário'}{p?.model && <span className="model-name">/ {p.model}</span>}</span><span><ShieldCheck size={13} />{c?.provider === 'local' ? 'Tradução no computador' : p && localEndpoint(p.baseUrl) ? 'Endpoint local' : p ? 'Endpoint remoto' : 'Privacidade por padrão'}</span></footer>
  </main>;
}
