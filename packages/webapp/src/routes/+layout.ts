import type { AppStoresInterface, ConfigSource } from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';
import settings from '$lib/settings-12-11-24.json';

export interface LayoutData {
	stores: AppStoresInterface;
}

export const load = () => {
	const settingsStore = writable<ConfigSource | undefined>(settings);

	return {
		stores: {
			settings: settingsStore,
			activeSubgraphs: writable<Record<string, string>>({}),
			accounts: derived(settingsStore, ($settings) => $settings?.accounts),
			activeAccountsItems: writable<Record<string, string>>({}),
			activeOrderStatus: writable<boolean | undefined>(undefined),
			orderHash: writable<string>(''),
			hideZeroBalanceVaults: writable<boolean>(false)
		}
	};
};

export const ssr = false;
