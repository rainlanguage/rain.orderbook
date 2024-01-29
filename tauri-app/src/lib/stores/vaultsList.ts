import { get, writable } from 'svelte/store';
import type { TokenVault as VaultsListItem } from '$lib/typeshare/vaults';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';

function useVaultsListStore() {
  const STORAGE_KEY = "vaults.vaultsList";

  const { subscribe, set } = writable<Array<VaultsListItem>>(localStorage.getItem(STORAGE_KEY) ? JSON.parse(localStorage.getItem(STORAGE_KEY) as string) : []);

  subscribe(value => {
    if(value) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([]));
    }
  });

  async function refetch() {
    const res: Array<VaultsListItem> = await invoke("vaults_list", {subgraphArgs: { url: get(subgraphUrl)} });
    set(res);
  }

  return {
    subscribe,
    refetch
  }
}

export const vaultsList = useVaultsListStore();