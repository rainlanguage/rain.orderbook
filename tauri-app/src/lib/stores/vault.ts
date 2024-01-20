import { get, writable } from 'svelte/store';
import type { Vault as VaultsListItem } from '../../types/vaults';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from './settings';

function useVaultsListStore() {
  const VAULTS_LIST_KEY = "vaults.vaultsList";

  const { subscribe, set } = writable<Array<VaultsListItem>>(localStorage.getItem(VAULTS_LIST_KEY) ? JSON.parse(localStorage.getItem(VAULTS_LIST_KEY) as string) : []);

  subscribe(value => {
    if(value) {
      localStorage.setItem(VAULTS_LIST_KEY, JSON.stringify(value));
    } else {
      localStorage.setItem(VAULTS_LIST_KEY, JSON.stringify([]));
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

export const vaults = useVaultsListStore();