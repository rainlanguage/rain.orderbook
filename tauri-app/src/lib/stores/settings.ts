import { isUrlValid } from '$lib/utils/url';
import { writable, derived } from 'svelte/store';
import every from 'lodash/every';
import { isAddress } from 'viem';
import { updateChainId } from '$lib/stores/chain';

export const rpcUrl = writable(localStorage.getItem("settings.rpcUrl") || '');
export const subgraphUrl = writable(localStorage.getItem("settings.subgraphUrl") || '');
export const orderbookAddress = writable(localStorage.getItem("settings.orderbookAddress") || '');
export const walletAddress = writable(localStorage.getItem("settings.walletAddress") || '');
export const walletDerivationIndex = writable(parseInt(localStorage.getItem("settings.walletDerivationIndex") || '0'));
export const forkBlockNumber = writable(parseInt(localStorage.getItem("settings.forkBlockNumber") || '45122616'));

rpcUrl.subscribe(value => {
  localStorage.setItem("settings.rpcUrl", value || '');
});
subgraphUrl.subscribe(value => {
  localStorage.setItem("settings.subgraphUrl", value || '');
});
orderbookAddress.subscribe(value => {
  localStorage.setItem("settings.orderbookAddress", value || '');
});
walletAddress.subscribe(value => {
  localStorage.setItem("settings.walletAddress", value || '');
});
walletDerivationIndex.subscribe(value => {
  localStorage.setItem("settings.walletDerivationIndex", (value || 0).toString());
});
forkBlockNumber.subscribe(value => {
  localStorage.setItem("settings.forkBlockNumber", (value || 45122616).toString());
});

export const isRpcUrlValid = derived(rpcUrl, (val) => isUrlValid(val));
export const isSubgraphUrlValid = derived(subgraphUrl, (val) => isUrlValid(val));
export const isOrderbookAddressValid = derived(orderbookAddress, (val) => isAddress(val));
export const isWalletAddressValid = derived(walletAddress, (val) => isAddress(val));

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