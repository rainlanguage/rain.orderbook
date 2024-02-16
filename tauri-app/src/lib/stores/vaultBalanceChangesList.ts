import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { listStore } from '$lib/storesGeneric/listStore';
import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';

export const useVaultBalanceChangesList = (id: string) =>  listStore<VaultBalanceChange>(
  `vaultBalanceChangesList-${id}`,
  (page) => invoke("vault_balance_changes_list", {subgraphArgs: { url: get(subgraphUrl).value}, id, paginationArgs: { page: page+1, page_size: 10 } }),
  (path) => invoke("vault_balance_changes_list_write_csv", {path, subgraphArgs: { url: get(subgraphUrl).value}, id})
);
