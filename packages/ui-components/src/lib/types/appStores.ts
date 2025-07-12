import type { Readable, Writable } from 'svelte/store';
import type { AccountCfg, Address, Hex, NewConfig } from '@rainlanguage/orderbook';

export interface AppStoresInterface {
	settings: Writable<NewConfig>;
	selectedChainIds: Writable<number[]>;
	accounts: Readable<Record<string, AccountCfg>>;
	activeAccountsItems: Writable<Record<string, Address>> | undefined;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<Hex>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeTokens: Writable<Address[]>;
	showMyItemsOnly: Writable<boolean>;
}
