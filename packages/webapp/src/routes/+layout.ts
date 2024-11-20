import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';
import settings from '$lib/settings-12-11-24.json';

export interface LayoutData {
	stores: AppStoresInterface;
}

export const load = () => {
	const settingsStore = writable(settings);
	
	return {
		stores: {
			settings: settingsStore,
			activeSubgraphs: writable<Record<string, string>>({}),
			accounts: derived(settingsStore, $settings => $settings.accounts),
			activeAccountsItems: writable<Record<string, string>>({}),
			activeOrderStatus: writable<boolean | undefined>(undefined)
		}
	};
};

export const ssr = false;
