import type { TokenVault } from '$lib/typeshare/vaultDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';

export const vaultDetail = detailStore<TokenVault>("vaults.vaultsDetail", async (id: string) => {
  const url = await subgraphUrl.load();
  return invoke("vault_detail", {id, subgraphArgs: { url } });
});