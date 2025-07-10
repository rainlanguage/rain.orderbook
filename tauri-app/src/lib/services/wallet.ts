import { invoke } from '@tauri-apps/api';
import type { Hex } from 'viem';
import { getOrderbookByChainId } from '$lib/utils/getOrderbookByChainId';
import { walletConnectNetwork } from '$lib/stores/walletconnect';
import { get } from 'svelte/store';

export const getAddressFromLedger = async (derivationIndex: number): Promise<Hex> => {
  const chainId = get(walletConnectNetwork);
  const orderbook = getOrderbookByChainId(chainId);
  return invoke('get_address_from_ledger', {
    derivationIndex,
    chainId,
    rpcs: orderbook.network.rpcs,
  });
};
