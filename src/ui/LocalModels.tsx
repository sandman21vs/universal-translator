import { useEffect, useState } from 'react';
import { Download, LoaderCircle, Check, RefreshCw } from 'lucide-react';
import { command, message } from '../bridge';

interface Model { id: string; name: string; source: string; target: string; version: string; url: string; bytes: number; sha256: string; license: string; licenseUrl: string; licenseEvidence: string; attribution: string; installed: boolean }
export default function LocalModels({ source, target }: { source: string; target: string }) {
  const [models, setModels] = useState<Model[]>([]);
  const [busy, setBusy] = useState<string | null>('status');
  const [notice, setNotice] = useState('');
  async function refresh() { const result = await command<{ models: Model[] }>('local_engine_status'); setModels(result.models); }
  useEffect(() => { let alive = true; void command<{ models: Model[] }>('local_engine_status').then(result => { if (alive) setModels(result.models); }).catch(e => { if (alive) setNotice(message(e)); }).finally(() => { if (alive) setBusy(null); }); return () => { alive = false; }; }, []);
  async function install(model: Model) {
    setBusy(model.id); setNotice(`Baixando ${model.name}. Aguarde a verificação e instalação…`);
    try { await command('install_local_model', { modelId: model.id }); await refresh(); setNotice(`${model.name} instalado. A tradução funciona sem internet.`); }
    catch (e) { setNotice(message(e)); } finally { setBusy(null); }
  }
  const from = source.split('-')[0], to = target.split('-')[0];
  const has = (a: string, b: string) => models.some(m => m.source === a && m.target === b && m.installed);
  const route = from === 'auto' ? 'Escolha o idioma de origem. A detecção automática ainda não está disponível neste motor.' : from === to ? 'Origem e destino são o mesmo idioma.' : has(from, to) ? `Par direto instalado: ${from} → ${to}.` : has(from, 'en') && has('en', to) ? `Tradução via inglês: ${from} → en → ${to}. Essa etapa intermediária pode reduzir a qualidade.` : `Par ${from} → ${to} ainda não instalado. Para português ↔ alemão, instale os dois modelos da rota via inglês.`;
  const direct = models.filter(m => m.source === from && m.target === to);
  const needed = direct.length ? direct : models.filter(m => (m.source === from && m.target === 'en') || (m.source === 'en' && m.target === to));
  const others = models.filter(m => !needed.includes(m));
  const row = (model: Model) => <div key={model.id} className="local-model">
    <div className="button-row"><strong>{model.name}</strong><button className="secondary" disabled={!!busy || model.installed} onClick={() => void install(model)}>{busy === model.id ? <LoaderCircle size={14} className="spin" /> : model.installed ? <Check size={14} /> : <Download size={14} />}{model.installed ? 'Instalado' : `Baixar · ${(model.bytes / 1_000_000).toFixed(1)} MB`}</button></div>
    <details><summary>Origem, licença e verificação</summary><p className="help">Versão {model.version} · {model.license}<br />{model.url}<br />Tamanho: {model.bytes.toLocaleString('pt-BR')} bytes<br />SHA-256: <code>{model.sha256}</code><br />Licença: {model.licenseUrl}<br />Evidência: {model.licenseEvidence}<br />{model.attribution}</p></details>
  </div>;
  return <div className="group models" aria-label="Modelos locais">
    <p role="status">{route}</p>
    {needed.map(row)}
    {others.length > 0 && <details className="advanced"><summary>Outros idiomas</summary>{others.map(row)}</details>}
    <button className="text-button" disabled={!!busy} onClick={() => { setBusy('status'); void refresh().catch(e => setNotice(message(e))).finally(() => setBusy(null)); }}><RefreshCw size={12} />Atualizar lista</button>
    {notice && <p role="status" className="notice">{notice}</p>}
  </div>;
}
