import type { Readable, Writable } from 'svelte/store';
import type { AccountCfg, Address, Hex } from '@rainlanguage/orderbook';

export interface AppStoresInterface {
	selectedChainIds: Writable<number[]>;
	accounts: Readable<Record<string, AccountCfg>>;
	activeAccountsItems: Writable<Record<string, Address>> | undefined;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<Hex>;
	hideZeroBalanceVaults: Writable<boolean>;
	hideInactiveOrdersVaults: Writable<boolean>;
	activeTokens: Writable<Address[]>;
	showMyItemsOnly: Writable<boolean>;
}
