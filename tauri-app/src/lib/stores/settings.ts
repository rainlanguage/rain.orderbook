import { asyncDerived, derived, get } from '@square/svelte-store';
import { cachedWritableInt, cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';
import  find from 'lodash/find';
import * as chains from 'viem/chains';
import { textFileStore } from '$lib/storesGeneric/textFileStore';
import { invoke } from '@tauri-apps/api';
import { type AppSettings, type ChainSettings } from '$lib/typeshare/appSettings';
import { getBlockNumberFromRpc, getChainIdFromRpc } from '$lib/services/chain';

interface ChainSettingsExtended extends ChainSettings {
  chain_id: number,
}

interface AppSettingsExtended {
  chains: Array<ChainSettingsExtended>;
}

// general
export const settingsText = cachedWritableStore<string>('settings', "", (s) => s, (s) => s);
export const settingsFile = textFileStore('Orderbook Settings Yaml', ['yml', 'yaml'], get(settingsText));
export const settings = asyncDerived(settingsText, async ($settingsText): Promise<AppSettingsExtended> => {
  const data: AppSettings = await invoke("parse_settings", {text: $settingsText});
  const chains = await Promise.all(data.chains.map(async (c): Promise<ChainSettingsExtended> => {
    const chainId: number = await getChainIdFromRpc(c.rpc_url);
    return {
      ...c,
      chain_id: chainId,
    };
  }));
  return { chains };
});

// chain
export const activeChainSettingsIndex = cachedWritableInt("settings.activeChainIndex", 0);
export const activeChainSettings = derived([settings, activeChainSettingsIndex], ([$settingsData, $activeChainSettingsIndex]) => $settingsData?.chains[$activeChainSettingsIndex]);
export const rpcUrl = derived(activeChainSettings, ($activeChainSettings) => $activeChainSettings?.rpc_url);
export const chainId = derived(activeChainSettings, ($activeChainSettings) => $activeChainSettings?.chain_id);
export const activeChain = derived(chainId, ($activeChainId) => find(Object.values(chains), (c) => c.id === $activeChainId));
export const activeChainHasBlockExplorer = derived(activeChain, ($activeChain) => {
  return $activeChain && $activeChain.blockExplorers?.default !== undefined;
});
export const activeChainLatestBlockNumber = derived(activeChainSettings, ($activeChainSettings) => getBlockNumberFromRpc($activeChainSettings.rpc_url));

// orderbook
export const activeOrderbookSettingsIndex = cachedWritableInt("settings.activeOrderbookIndex", 0);
export const activeOrderbookSettings =  derived([activeChainSettings, activeOrderbookSettingsIndex], ([$activeChainSettings, $activeOrderbookSettingsIndex]) => $activeChainSettings?.orderbooks[$activeOrderbookSettingsIndex]);
export const subgraphUrl = derived(activeOrderbookSettings, ($activeOrderbookSettings) => $activeOrderbookSettings?.subgraph_url);
export const orderbookAddress = derived(activeOrderbookSettings, ($activeOrderbookSettings) => $activeOrderbookSettings?.address);

export const hasRequiredSettings = derived([activeChainSettings, activeOrderbookSettings], ([$activeChainSettings, $activeOrderbookSettings]) => $activeChainSettings !== undefined && $activeOrderbookSettings !== undefined);

// When settings data updated, reset active chain
settings.subscribe((val) => {
  if(val && val.chains.length < get(activeChainSettingsIndex)) {
    activeChainSettingsIndex.set(0);
  }
});

// When active chain updated, reset active orderbook
activeChainSettings.subscribe(async ()  => {
  activeOrderbookSettingsIndex.set(0);
});
