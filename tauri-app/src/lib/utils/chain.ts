import { get } from 'svelte/store';
import { rpcUrl } from '$lib/stores/settings';
import { invoke } from '@tauri-apps/api';

export const getChainIdFromRpc = async (): Promise<number> => invoke('get_chainid', {rpcUrl: get(rpcUrl).value});