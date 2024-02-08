import type { TokenVault } from '$lib/typeshare/vaultDetail';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { subgraphUrl } from '$lib/stores/settings';
import { detailStore } from '$lib/storesGeneric/detailStore';

export const vaultDetail = detailStore<TokenVault>("vaults.vaultsDetail", (id: string) => invoke("vault_detail", {id, subgraphArgs: { url: get(subgraphUrl)} }));