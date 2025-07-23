import { invoke } from '@tauri-apps/api';
import type { Hex } from 'viem';
import { getNetworkByChainId } from '$lib/utils/raindexClient/getNetworkByChainId';
import { walletConnectNetwork } from '$lib/stores/walletconnect';
import { get } from 'svelte/store';

export const getAddressFromLedger = async (derivationIndex: number): Promise<Hex> => {
  const chainId = get(walletConnectNetwork);
  const network = getNetworkByChainId(chainId);
  return invoke('get_address_from_ledger', {
    derivationIndex,
    chainId,
    rpcs: network.rpcs,
  });
};
