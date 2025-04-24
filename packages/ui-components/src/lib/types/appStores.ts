import type { Readable, Writable } from 'svelte/store';
import type { AccountCfg, Config, OrderbookCfg, SubgraphCfg } from '@rainlanguage/orderbook';
export interface AppStoresInterface {
	settings: Writable<Config>;
	activeSubgraphs: Writable<Record<string, SubgraphCfg>>;
	accounts: Readable<Record<string, AccountCfg>>;
	activeAccountsItems: Writable<Record<string, string>>;
	activeOrderStatus: Writable<boolean | undefined>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeNetworkRef: Writable<string | undefined>;
	activeOrderbookRef: Writable<string | undefined>;
	// New ones
	activeOrderbook: Readable<OrderbookCfg | undefined>;
	subgraph: Readable<SubgraphCfg | undefined>;
	activeAccounts: Readable<{
		[k: string]: AccountCfg;
	}>;
	showMyItemsOnly: Writable<boolean>;
}
