import type { KeyboardEvent, RefObject } from 'react';
import { Check, Copy, LoaderCircle, Settings2 } from 'lucide-react';
import { command } from '../bridge';
import type { Snapshot } from '../session';
import type { Config } from '../types';

interface Props {
  state: Snapshot; config?: Config; editor: RefObject<HTMLTextAreaElement | null>;
  selection: boolean; canInsert: boolean; available: boolean; copied: boolean; notice: string;
  onText: (value: string) => void; onOutput: (value: string) => void;
  onClose: () => void; onSettings: () => void; onCopy: () => void; onInsert: () => void; onRetry: () => void;
}
export default function CompactOverlay({ state, config, editor, selection, canInsert, available, copied, notice, onText, onOutput, onClose, onSettings, onCopy, onInsert }: Props) {
  const working = state.phase === 'translating' || state.phase === 'inserting';
  const locked = state.phase === 'inserting';
  const error = state.error || notice;
  const language = (value?: string) => value === 'auto' ? 'AUTO' : (value || '').split('-')[0].toUpperCase();
  const pivot = config?.provider === 'local' && config.source !== 'auto' && config.source.split('-')[0] !== 'en' && config.target.split('-')[0] !== 'en' && config.source.split('-')[0] !== config.target.split('-')[0];
  const drag = (e: { button: number }) => { if (e.button === 0) void command('drag_overlay').catch(() => {}); };
  // Enter delivers the translation: into the original field when there is one, otherwise to the clipboard.
  const deliver = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.nativeEvent.isComposing || e.key !== 'Enter' || e.shiftKey) return;
    e.preventDefault();
    if (canInsert) onInsert(); else if (!available && state.output && !locked) onCopy();
  };
  return <main className="compact-overlay" aria-label="Painel rápido de tradução" onKeyDown={e => { if (e.key === 'Escape' && !locked) { e.preventDefault(); onClose(); } }}>
    <div className="compact-row">
      <span className="compact-lang" title={`${config?.source || ''} · arraste para mover`} onMouseDown={drag}>{language(config?.source)}</span>
      <textarea id="quick-source" ref={editor} aria-label={selection ? 'Texto selecionado' : 'Seu texto'} value={state.text} placeholder="Digite aqui…" readOnly={selection} disabled={!config || locked} onChange={e => onText([...e.target.value].slice(0,8000).join(''))} onKeyDown={deliver} />
      <button className="compact-icon" aria-label="Abrir configurações" title="Configurações" disabled={locked || !config} onClick={onSettings}><Settings2 size={15} /></button>
    </div>
    <div className="compact-row">
      <span className="compact-lang" title={`${config?.target || ''}${pivot ? ' · via inglês' : ''} · arraste para mover`} onMouseDown={drag}>{language(config?.target)}</span>
      <textarea id="quick-result" aria-label="Tradução" value={state.output} placeholder={working ? 'Traduzindo…' : 'Tradução'} disabled={state.phase !== 'ready'} onChange={e => onOutput(e.target.value)} onKeyDown={deliver} />
      <button className="compact-icon" aria-label="Copiar tradução" title={available ? 'Copiar · Enter insere no campo original' : 'Copiar · Enter'} disabled={!state.output || locked} onClick={onCopy}>{working ? <LoaderCircle className="spin" size={15} /> : copied ? <Check size={15} /> : <Copy size={15} />}</button>
    </div>
    {error && <div role="alert" className="compact-error">{error}</div>}
  </main>;
}
