import { isUrlValid } from '$lib/utils/url';
import { writable, derived } from 'svelte/store';

export const rpcUrl = writable(localStorage.getItem("settings.rpcUrl") || '');
export const subgraphUrl = writable(localStorage.getItem("settings.subgraphUrl") || '');

rpcUrl.subscribe(value => {
  localStorage.setItem("settings.rpcUrl", value || '');
});
subgraphUrl.subscribe(value => {
  localStorage.setItem("settings.subgraphUrl", value || '');
});

export const isRpcUrlValid = derived(rpcUrl, ($rpcUrl) => isUrlValid($rpcUrl));
export const isSubgraphUrlValid = derived(subgraphUrl, ($rpcUrl) => isUrlValid($rpcUrl));

export const isSettingsDefined = derived([rpcUrl, subgraphUrl], ([$rpcUrl, $subgraphUrl]) => $rpcUrl && $rpcUrl.trim().length > 0 && $subgraphUrl && $subgraphUrl.trim().length > 0);
export const isSettingsValid = derived([isRpcUrlValid, isSubgraphUrlValid], ([$isRpcUrlValid, $isSubgraphUrlValid]) => $isRpcUrlValid && $isSubgraphUrlValid);
export const isSettingsDefinedAndValid = derived([isSettingsDefined, isSettingsValid], ([$isSettingsDefined, $isSettingsValid]) => $isSettingsDefined && $isSettingsValid);