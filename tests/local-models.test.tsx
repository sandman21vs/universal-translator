import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
const mocks = vi.hoisted(() => ({ command: vi.fn() }));
vi.mock('../src/bridge', () => ({ command: mocks.command, message: String }));
import LocalModels from '../src/ui/LocalModels';
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it('lists without downloading; explicit click installs only the selected pair', async () => {
  const model = {id:'pt-en-1.9',name:'Português → Inglês',source:'pt',target:'en',version:'1.9',url:'https://argos-net.com/test',bytes:69447231,sha256:'checksum',license:'CC-BY-4.0',licenseUrl:'https://creativecommons.org/licenses/by/4.0/',attribution:'OPUS-MT',installed:false};
  mocks.command.mockImplementation(async (name: string) => name === 'local_engine_status' ? {models:[model]} : (model.installed=true, undefined));
  render(<LocalModels source="pt-BR" target="en-US" />);
  await screen.findByText('Português → Inglês');
  expect(mocks.command.mock.calls.some(([name]) => name === 'install_local_model')).toBe(false);
  fireEvent.click(screen.getByRole('button', {name:/Baixar/}));
  await waitFor(() => expect(mocks.command).toHaveBeenCalledWith('install_local_model', {modelId:'pt-en-1.9'}));
  await screen.findByText('Par direto instalado: pt → en.');
  expect((screen.getByRole('button', {name:'Instalado'}) as HTMLButtonElement).disabled).toBe(true);
});
it('discloses a pivot route when only the two intermediate models exist', async () => {
  mocks.command.mockResolvedValue({models:[{id:'pt-en',name:'PT → EN',source:'pt',target:'en',installed:true,bytes:10},{id:'en-de',name:'EN → DE',source:'en',target:'de',installed:true,bytes:10}]});
  render(<LocalModels source="pt-BR" target="de-CH" />);
  await screen.findByText(/Tradução via inglês: pt → en → de/);
});
