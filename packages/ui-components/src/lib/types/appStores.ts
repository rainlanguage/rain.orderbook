import type { Readable, Writable } from 'svelte/store';
import type {
	AccountCfg,
	NewConfig,
	OrderbookCfg,
	SubgraphCfg,
	SgTokenAddress
} from '@rainlanguage/orderbook';

export interface AppStoresInterface {
	settings: Writable<NewConfig>;
	activeSubgraphs: Writable<Record<string, SubgraphCfg>>;
	accounts: Readable<Record<string, AccountCfg>>;
	activeAccountsItems: Writable<Record<string, string>> | undefined;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeNetworkRef: Writable<string | undefined>;
	activeOrderbookRef: Writable<string | undefined>;
	activeOrderbook: Readable<OrderbookCfg | undefined>;
	subgraph: Readable<SubgraphCfg | undefined>;
	showMyItemsOnly: Writable<boolean>;
	activeNetworkOrderbooks: Readable<Record<string, OrderbookCfg>>;
	activeTokens: Writable<SgTokenAddress[]>;
}
