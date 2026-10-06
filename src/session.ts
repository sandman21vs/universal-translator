// src/session.ts
import type { TranslationResult } from './types';
export type Phase = 'idle' | 'debouncing' | 'translating' | 'ready' | 'inserting' | 'error';
export interface Snapshot { text: string; output: string; phase: Phase; error: string; result: TranslationResult | null }
export interface SessionApi {
  translate(id: number, text: string): Promise<TranslationResult>;
  cancel(id: number): Promise<unknown>;
  insert(id: number, text: string): Promise<unknown>;
}
export class TranslationSession {
  private state: Snapshot = { text: '', output: '', phase: 'idle', error: '', result: null };
  private listeners = new Set<() => void>();
  private sequence = Date.now() * 1000;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private lastText = '';
  constructor(private api: SessionApi, private delay = 500) {}
  snapshot = () => this.state;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  private publish(next: Partial<Snapshot>) { this.state = { ...this.state, ...next }; this.listeners.forEach(fn => fn()); }
  private invalidate() { clearTimeout(this.timer); const id = ++this.sequence; void this.api.cancel(id).catch(() => {}); return id; }
  configure(delay: number) { this.delay = delay; this.cancel(); }
  updateText(text: string) {
    if (this.state.phase === 'inserting') return;
    this.invalidate();
    this.publish({ text, output: '', result: null, error: '', phase: text.trim() ? 'debouncing' : 'idle' });
    if (text.trim()) this.timer = setTimeout(() => { void this.run(); }, this.delay);
  }
  async run() {
    clearTimeout(this.timer);
    if (!this.state.text.trim() || this.state.phase === 'inserting') return;
    const id = ++this.sequence;
    const text = this.state.text;
    this.publish({ phase: 'translating', result: null, error: '' });
    try {
      const result = await this.api.translate(id, text);
      if (id !== this.sequence || result.id !== id) return;
      this.lastText = text;
      this.publish({ phase: 'ready', output: result.text, result });
    } catch (error) { if (id === this.sequence) this.publish({ phase: 'error', error: String(error instanceof Error ? error.message : error), result: null }); }
  }
  editOutput(output: string) { if (this.state.phase === 'ready') this.publish({ output }); }
  canInsert() { return this.state.phase === 'ready' && !!this.state.result && !!this.state.output.trim(); }
  async insert() {
    if (!this.canInsert()) return false;
    const result = this.state.result!;
    const output = this.state.output;
    this.publish({ phase: 'inserting' });
    try {
      await this.api.insert(result.id, output);
      this.invalidate();
      this.publish({ text: '', output: '', result: null, error: '', phase: 'idle' });
      return true;
    } catch (error) {
      this.publish({ phase: 'error', result: null, error: String(error instanceof Error ? error.message : error) });
      return false;
    }
  }
  cancel() { if (this.state.phase === 'inserting') return; this.invalidate(); this.publish({ phase: 'idle', result: null }); }
  resume() { this.updateText(this.state.text); }
  repeat() { if (this.lastText) this.updateText(this.lastText); }
  clear() { this.lastText = ''; this.updateText(''); }
}
