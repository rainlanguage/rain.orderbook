import { invoke } from '@tauri-apps/api';
import type { Hex } from 'viem';

export const getAddressFromLedger = async (
  chainId: number,
  rpcs: string[],
  derivationIndex: number,
): Promise<Hex> => {
  return invoke('get_address_from_ledger', {
    derivationIndex,
    chainId,
    rpcs,
  });
};
