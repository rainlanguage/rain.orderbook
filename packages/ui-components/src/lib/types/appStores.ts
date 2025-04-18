import type { Readable, Writable } from 'svelte/store';
import type { Config, OrderbookCfg, SubgraphCfg } from '@rainlanguage/orderbook';
export interface AppStoresInterface {
	settings: Writable<Config | undefined>;
	activeSubgraphs: Writable<Record<string, SubgraphCfg>>;
	accounts: Readable<Record<string, string>>;
	activeAccountsItems: Writable<Record<string, string>>;
	activeOrderStatus: Writable<boolean | undefined>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeNetworkRef: Writable<string | undefined>;
	activeOrderbookRef: Writable<string | undefined>;
	// New ones
	activeOrderbook: Readable<OrderbookCfg | undefined>;
	subgraphUrl: Readable<string | undefined>;
	activeAccounts: Readable<{
		[k: string]: string;
	}>;
	showMyItemsOnly: Writable<boolean>;
}
