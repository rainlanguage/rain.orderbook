import type { Readable, Writable } from 'svelte/store';
import type { ConfigSource, OrderbookConfigSource } from '@rainlanguage/orderbook';
export interface AppStoresInterface {
	settings: Writable<ConfigSource | undefined>;
	activeSubgraphs: Writable<Record<string, string>>;
	accounts: Readable<Record<string, string>>;
	activeAccountsItems: Writable<Record<string, string>>;
	activeOrderStatus: Writable<boolean>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeNetworkRef: Writable<string | undefined>;
	activeOrderbookRef: Writable<string | undefined>;
	// New ones
	activeOrderbook: Readable<OrderbookConfigSource | undefined>;
	subgraphUrl: Readable<string | undefined>;
	activeAccounts: Readable<{
		[k: string]: string;
	}>;
	showMyItemsOnly: Writable<boolean>;
}
