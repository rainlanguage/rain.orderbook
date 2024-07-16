import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';
import type { Vault } from '$lib/typeshare/vaultsList';

export const vaultDetail = detailStore<Vault>('vaults.vaultsDetail', async (id: string) => {
  const url = await subgraphUrl.load();
  return invoke('vault_detail', { id, subgraphArgs: { url } });
});
