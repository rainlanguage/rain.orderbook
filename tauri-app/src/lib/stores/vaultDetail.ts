import { get, writable } from 'svelte/store';
import type { TokenVault } from '$lib/typeshare/vaultDetail';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';

function useVaultDetailStore() {
  const STORAGE_KEY = "vaults.vaultsDetail";

  const { subscribe, update } = writable<{[id: string]: TokenVault}>(localStorage.getItem(STORAGE_KEY) ? JSON.parse(localStorage.getItem(STORAGE_KEY) as string) : {});

  subscribe(value => {
    if(value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({}));
    }
  });

  async function refetch(id: string) {
    const res: TokenVault = await invoke("vault_detail", {id, subgraphArgs: { url: get(subgraphUrl)} });
    update((value) => {
      return {... value, [id]: res};
    });
  }

  return {
    subscribe,
    refetch
  }
}

export const vaultDetail = useVaultDetailStore();