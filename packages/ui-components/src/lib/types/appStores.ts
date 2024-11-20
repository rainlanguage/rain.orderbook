import type { Writable } from 'svelte/store';

export interface AppStoresInterface {
	settings: Writable<Record<string, string>>;
	activeSubgraphs: Writable<Record<string, string>>;
	accounts: Writable<Record<string, string>>;
	activeAccountsItems: Writable<Record<string, string>>;
  activeOrderStatus: Writable<boolean | undefined>;
}
