import { get } from 'svelte/store';
import { rpcUrl } from '$lib/stores/settings';
import { invoke } from '@tauri-apps/api';

export const getForkBlockNumberFromRpc = async (): Promise<number> => invoke('get_block_number', {rpcUrl: get(rpcUrl)});
