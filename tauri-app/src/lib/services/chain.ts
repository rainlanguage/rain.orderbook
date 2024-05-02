import { invoke } from '@tauri-apps/api';

export const getChainIdFromRpc = async (rpcUrl: string): Promise<number> =>
  invoke('get_chainid', { rpcUrl });

export const getBlockNumberFromRpc = async (rpcUrl: string): Promise<number> =>
  invoke('get_block_number', { rpcUrl });
