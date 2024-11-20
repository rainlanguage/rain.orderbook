import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { writable } from 'svelte/store';
import settings from '$lib/settings-12-11-24.json';

export interface LayoutData {
	stores: AppStoresInterface;
}

export const load = () => {
	return {
		stores: {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			settings: writable<any>(settings),
			activeSubgraphs: writable<Record<string, string>>({}),
			accounts: writable<Record<string, string>>(settings.accounts),
			activeAccountsItems: writable<Record<string, string>>({})
		}
	};
};

export const ssr = false;
