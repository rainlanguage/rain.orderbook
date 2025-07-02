import { invoke } from '@tauri-apps/api';

export const getChainIdFromRpc = async (rpcs: string[]): Promise<number> =>
  invoke('get_chainid', { rpcs });

export const getBlockNumberFromRpc = async (rpcs: string[]): Promise<number> =>
  invoke('get_block_number', { rpcs });
