import type { Readable, Writable } from 'svelte/store';
import type { AccountCfg, Address, NewConfig } from '@rainlanguage/orderbook';

export interface AppStoresInterface {
	settings: Writable<NewConfig>;
	selectedChainIds: Writable<number[]>;
	accounts: Readable<Record<string, AccountCfg>>;
	activeAccountsItems: Writable<Record<string, Address>> | undefined;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeAccounts: Readable<{
		[k: string]: AccountCfg;
	}>;
	showMyItemsOnly: Writable<boolean>;
}
