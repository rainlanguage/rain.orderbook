import type { Readable, Writable } from 'svelte/store';
import type { ConfigSource, OrderbookConfigSource, OrderbookCfgRef } from '@rainlanguage/orderbook';

export interface AppStoresInterface {
	settings: Writable<ConfigSource | undefined>;
	activeSubgraphs: Writable<Record<string, string>>;
	accounts: Readable<Record<string, string>> | undefined;
	activeAccountsItems: Writable<Record<string, string>> | undefined;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeNetworkRef: Writable<string | undefined>;
	activeOrderbookRef: Writable<string | undefined>;
	activeOrderbook: Readable<OrderbookConfigSource | undefined>;
	subgraphUrl: Readable<string | undefined>;
	activeAccounts: Readable<{
		[k: string]: string;
	}>;
	showMyItemsOnly: Writable<boolean>;
	activeNetworkOrderbooks: Readable<Record<OrderbookCfgRef, OrderbookConfigSource>>;
}
