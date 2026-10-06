// src/bridge.ts
import { invoke, isTauri } from '@tauri-apps/api/core';
export async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error('Abra o aplicativo desktop com npm run desktop:dev. O navegador mostra a interface; o backend seguro é Rust.');
  return invoke<T>(name, args);
}
export function message(error: unknown): string { return error instanceof Error ? error.message : String(error); }
