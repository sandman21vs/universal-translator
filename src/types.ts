// src/types.ts
export type ProviderId = 'local' | 'ollama' | 'openai' | 'anthropic' | 'google' | 'libretranslate';
export interface ProviderConfig { baseUrl: string; model: string; allowRemote: boolean; allowInsecure: boolean }
export interface Config {
  schemaVersion: number; provider: ProviderId; providers: Record<ProviderId, ProviderConfig>;
  source: string; target: string; tone: string; formality: string; relationship: string;
  debounceMs: number; timeoutSecs: number; temperature: number; promptExtra: string;
  composeShortcut: string; selectionShortcut: string; repeatShortcut: string;
  theme: 'system' | 'dark' | 'light'; autostart: boolean;
}
export interface TranslationResult { id: number; text: string; provider: string; model: string; elapsedMs: number; cached: boolean }
export interface Bootstrap { compact: boolean; config: Config; warning: string | null; shortcutError: string | null; platform: string; insertionAvailable: boolean; paused: boolean; capabilities: { styles: boolean; regionalVariants: boolean; streaming: boolean; protocol: string } }
export const languages = [
  ['pt-BR','Português · Brasil'], ['pt-PT','Português · Portugal'], ['en-US','English · US'], ['en-GB','English · UK'],
  ['de-DE','Deutsch · Deutschland'], ['de-CH','Deutsch · Schweiz'], ['es-ES','Español · España'], ['es-MX','Español · México'],
  ['es-AR','Español · Argentina'], ['es-419','Español · América Latina'], ['fr-FR','Français'], ['it-IT','Italiano'],
] as const;
export const providerNames: Record<ProviderId, string> = { local: 'Local integrado · modelos Argos', ollama: 'Ollama', openai: 'OpenAI / compatível', anthropic: 'Anthropic', google: 'Google Cloud', libretranslate: 'LibreTranslate (servidor externo)' };
export function localEndpoint(endpoint: string): boolean { try { const url=new URL(endpoint); return ['localhost','[::1]'].includes(url.hostname) || /^127\.(\d{1,3}\.){2}\d{1,3}$/.test(url.hostname); } catch { return false; } }
