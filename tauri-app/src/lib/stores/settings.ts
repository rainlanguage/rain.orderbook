import { writable, derived } from 'svelte/store';

export const rpcUrl = writable(localStorage.getItem("settings.rpcUrl") || '');

rpcUrl.subscribe(value => {
  localStorage.setItem("settings.rpcUrl", value || '');
});

export const subgraphUrl = writable(localStorage.getItem("settings.subgraphUrl") || '');

subgraphUrl.subscribe(value => {
  localStorage.setItem("settings.subgraphUrl", value || '');
});

export const isSettingsDefined = derived([rpcUrl, subgraphUrl], ([$rpcUrl, $subgraphUrl]) => $rpcUrl && $rpcUrl.trim().length > 0 && $subgraphUrl && $subgraphUrl.trim().length > 0);