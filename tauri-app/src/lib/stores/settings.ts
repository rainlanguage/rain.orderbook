import { isUrlValid } from '$lib/utils/url';
import { derived } from 'svelte/store';
import every from 'lodash/every';
import { isAddress } from 'viem';
import { validatedStringStore } from '$lib/storesGeneric/settingStore';
import { cachedWritableInt } from '$lib/storesGeneric/cachedWritableStore';
import  find from 'lodash/find';
import * as chains from 'viem/chains'
import { setChainIdFromRpc } from '$lib/utils/chain';

const BLANK_WALLET_ADDRESS = '';

export const rpcUrl = validatedStringStore("settings.rpcUrl", '', isUrlValid);
export const subgraphUrl = validatedStringStore("settings.subgraphUrl", '', isUrlValid);
export const orderbookAddress = validatedStringStore("settings.orderbookAddress", '', isAddress);
export const walletAddress = validatedStringStore("settings.walletAddress", BLANK_WALLET_ADDRESS, isAddress);
export const walletDerivationIndex = cachedWritableInt("settings.walletDerivationIndex", 0);
export const forkBlockNumber = cachedWritableInt("settings.forkBlockNumber", 1);
export const chainId = cachedWritableInt("settings.chainId", 0)

export const activeChain = derived(chainId, ($chainId) => find(Object.values(chains), (c) => c.id === $chainId));
export const activeChainHasBlockExplorer = derived(activeChain, ($activeChain) => {
  return $activeChain && $activeChain.blockExplorers?.default !== undefined;
})
export const allRequiredSettingsValid = derived([rpcUrl, subgraphUrl, orderbookAddress], (vals) => every(vals.map((v) => v.isValid)));
export const walletAddressMatchesOrBlank = derived(walletAddress, $walletAddress => {
  return (otherAddress: string) => $walletAddress.value === otherAddress || $walletAddress.value === BLANK_WALLET_ADDRESS;
});

rpcUrl.subscribe(value => {
  if(value.isValid) {
    setChainIdFromRpc();
  }
});
