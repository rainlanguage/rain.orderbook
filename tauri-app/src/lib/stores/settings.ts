import { isUrlValid } from '$lib/utils/url';
import { derived } from 'svelte/store';
import every from 'lodash/every';
import { isAddress } from 'viem';
import { updateChainId } from '$lib/stores/chain';
import { cachedWritableInt, cachedWritableString } from '$lib/storesGeneric/cachedWritable';

export const rpcUrl = cachedWritableString("settings.rpcUrl", '');
export const subgraphUrl = cachedWritableString("settings.subgraphUrl", '');
export const orderbookAddress = cachedWritableString("settings.orderbookAddress", '');
export const walletAddress = cachedWritableString("settings.walletAddress", '');
export const walletDerivationIndex = cachedWritableInt("settings.walletDerivationIndex", 0);
export const forkBlockNumber = cachedWritableInt("settings.forkBlockNumber", 53247376);

export const isRpcUrlValid = derived(rpcUrl, (val) => val  && isUrlValid(val));
export const isSubgraphUrlValid = derived(subgraphUrl, (val) => val && isUrlValid(val));
export const isOrderbookAddressValid = derived(orderbookAddress, (val) => val && isAddress(val));
export const isWalletAddressValid = derived(walletAddress, (val) => val && isAddress(val));

isRpcUrlValid.subscribe(value => {
  if(value) {
    updateChainId();
  }
})

export const isSettingsDefined = derived([rpcUrl, subgraphUrl, orderbookAddress], (vals) => every(vals.map((v) => v && v.trim().length > 0)));
export const isSettingsValid = derived([isRpcUrlValid, isSubgraphUrlValid], (vals) => every(vals));
export const isSettingsDefinedAndValid = derived([isSettingsDefined, isSettingsValid], (vals) => every(vals));

export const walletAddressMatchesOrBlank = derived(walletAddress, val => {
  return (otherAddress: string) => val === otherAddress || val === '';
});