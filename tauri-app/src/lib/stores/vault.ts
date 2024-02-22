import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';
import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';
import { detailStore } from '$lib/storesGeneric/detailStore';
import type { TokenVault } from '$lib/typeshare/vaultsList';
import { asyncDerived } from '@square/svelte-store';

export const vaultsList = asyncDerived(subgraphUrl, async ($subgraphUrl) => {
  await subgraphUrl.load();

  return listStore<TokenVault>(
    `${$subgraphUrl}.vaultsList`,
    (page) => invoke("vaults_list", {subgraphArgs: { url: $subgraphUrl }, paginationArgs: { page: page+1, page_size: 10 } }),
    (path) => invoke("vaults_list_write_csv", {path, subgraphArgs: { url: $subgraphUrl }}),
  );
});

export const vaultDetail = detailStore<TokenVault>("vaults.vaultsDetail", async (id: string) => {
  const url = await subgraphUrl.load();
  return invoke("vault_detail", {id, subgraphArgs: { url } });
});

export const useVaultBalanceChangesList = (id: string) => listStore<VaultBalanceChange>(
  `vaultBalanceChangesList-${id}`,
  async (page) => {
    const url = await subgraphUrl.load();
    return invoke("vault_balance_changes_list", {subgraphArgs: { url }, id, paginationArgs: { page: page+1, page_size: 10 } })
  },
  async (path) => {
    const url = await subgraphUrl.load();
    return invoke("vault_balance_changes_list_write_csv", {path, subgraphArgs: { url }, id})
  },
);