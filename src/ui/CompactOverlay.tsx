import type { RefObject } from 'react';
import { ArrowRight, Check, Clipboard, CornerDownLeft, GripHorizontal, LoaderCircle, RotateCcw, Settings2, X } from 'lucide-react';
import { command } from '../bridge';
import { ptBR } from '../i18n';
import type { Snapshot } from '../session';
import type { Config } from '../types';

interface Props {
  state: Snapshot; config?: Config; editor: RefObject<HTMLTextAreaElement | null>;
  selection: boolean; canInsert: boolean; available: boolean; copied: boolean; notice: string;
  onText: (value: string) => void; onOutput: (value: string) => void;
  onClose: () => void; onSettings: () => void; onCopy: () => void; onInsert: () => void; onRetry: () => void;
}
export default function CompactOverlay({ state, config, editor, selection, canInsert, available, copied, notice, onText, onOutput, onClose, onSettings, onCopy, onInsert, onRetry }: Props) {
  const working = state.phase === 'translating' || state.phase === 'inserting';
  const locked = state.phase === 'inserting';
  const error = state.error || notice;
  const language = (value?: string) => value === 'auto' ? 'AUTO' : (value || '').toUpperCase();
  const pivot = config?.provider === 'local' && config.source !== 'auto' && config.source.split('-')[0] !== 'en' && config.target.split('-')[0] !== 'en' && config.source.split('-')[0] !== config.target.split('-')[0];
  return <main className="compact-overlay" aria-label="Painel rápido de tradução" onKeyDown={e => { if (e.key === 'Escape' && !locked) { e.preventDefault(); onClose(); } }}>
    <header className="compact-header"><div className="compact-drag" title="Arraste para mover" onMouseDown={e => { if (e.button === 0) void command('drag_overlay').catch(() => {}); }}><GripHorizontal size={15} /><strong>{language(config?.source)} <ArrowRight size={13} />{language(config?.target)}</strong>{pivot && <span title="Tradução com idioma intermediário inglês">via EN</span>}</div><button className="icon-button" aria-label="Abrir configurações" title="Abrir configurações completas" disabled={locked || !config} onClick={onSettings}><Settings2 size={16} /></button><button className="icon-button" aria-label="Fechar painel" title="Fechar · Esc" disabled={locked} onClick={onClose}><X size={17} /></button></header>
    <div className="compact-editors"><div className="compact-field"><div className="compact-field-label"><label htmlFor="quick-source">{selection ? 'Texto selecionado' : 'Seu texto'}</label><span>{[...state.text].length}/8.000</span></div><textarea id="quick-source" ref={editor} value={state.text} placeholder="Digite aqui…" readOnly={selection} disabled={!config || locked} onChange={e => onText([...e.target.value].slice(0,8000).join(''))} onKeyDown={e => { if (e.nativeEvent.isComposing) return; if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); if (canInsert) onInsert(); } }} /></div>
      <div className={`compact-field compact-result ${state.phase === 'ready' ? 'ready' : ''}`}><div className="compact-field-label"><label htmlFor="quick-result">Tradução · revisável</label><span role="status">{working && <LoaderCircle className="spin" size={11} />}{ptBR[state.phase]}</span></div><textarea id="quick-result" value={state.output} placeholder="A tradução aparece aqui…" disabled={state.phase !== 'ready'} onChange={e => onOutput(e.target.value)} /></div></div>
    {error && <div role="alert" className="compact-error">{error}</div>}
    <div className="compact-actions"><button className="secondary" aria-label="Copiar tradução" title="Copiar tradução" disabled={!state.output || locked} onClick={onCopy}>{copied ? <Check size={14} /> : <Clipboard size={14} />}{copied ? 'Copiado' : 'Copiar'}</button><button className="icon-button" aria-label="Traduzir novamente" title="Traduzir novamente" disabled={!config || !state.text.trim() || locked} onClick={onRetry}><RotateCcw size={14} /></button><span className="compact-hint">Shift + Enter: nova linha</span><button className="primary" disabled={!canInsert} title={available ? 'Inserir no campo original sem enviar a mensagem' : 'Abra com o atalho sobre o campo de destino; ou copie'} onClick={onInsert}><CornerDownLeft size={14} />Inserir <kbd>Enter</kbd></button></div>
    <footer className="compact-footer"><span>{config?.provider === 'local' ? 'Local integrado' : config?.provider || 'Conectando…'}{state.result ? ` · ${state.result.elapsedMs} ms` : ''}</span><span>{selection ? 'Seleção · copiar' : available ? 'Insere sem enviar' : 'Sem destino · use Copiar'}</span></footer>
  </main>;
}
