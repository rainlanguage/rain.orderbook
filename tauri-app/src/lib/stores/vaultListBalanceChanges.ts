import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { usePaginatedCachedStore } from '$lib/storesGeneric/paginatedStore';
import type { VaultBalanceChange } from '$lib/typeshare/vaultListBalanceChanges';
export const useVaultListBalanceChanges = (id: string) =>  usePaginatedCachedStore<VaultBalanceChange>(
  `vaultListBalanceChanges-${id}`,
  (page) => invoke("vault_list_balance_changes", {subgraphArgs: { url: get(subgraphUrl)}, id, paginationArgs: { page, page_size: 10 } }),
  (path) => invoke("vault_list_balance_changes_write_csv", {path, subgraphArgs: { url: get(subgraphUrl)}, id, paginationArgs: { page: 1, page_size: 1000 } })
);
