import type { Readable, Writable } from 'svelte/store';
import type { ConfigSource } from '../typeshare/config';
export interface AppStoresInterface {
	settings: Writable<ConfigSource | undefined>;
	activeSubgraphs: Writable<Record<string, string>>;
	accounts: Readable<Record<string, string>>;
	activeAccountsItems: Writable<Record<string, string>>;
	activeOrderStatus: Writable<boolean | undefined>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
}
