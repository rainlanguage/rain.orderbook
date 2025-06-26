import type { Readable, Writable } from 'svelte/store';
import type {
	AccountCfg,
	Address,
	NetworkCfg,
	NewConfig,
	OrderbookCfg,
	SubgraphCfg
} from '@rainlanguage/orderbook';

export interface AppStoresInterface {
	settings: Writable<NewConfig>;
	selectedChainIds: Writable<number[]>;
	activeSubgraphs: Writable<Record<string, SubgraphCfg>>;
	activeNetworks: Writable<Record<string, NetworkCfg>>;
	accounts: Readable<Record<string, AccountCfg>>;
	activeAccountsItems: Writable<Record<string, Address>> | undefined;
	showInactiveOrders: Writable<boolean>;
	orderHash: Writable<string>;
	hideZeroBalanceVaults: Writable<boolean>;
	activeNetworkRef: Writable<string | undefined>;
	activeOrderbookRef: Writable<string | undefined>;
	activeOrderbook: Readable<OrderbookCfg | undefined>;
	subgraph: Readable<SubgraphCfg | undefined>;
	activeAccounts: Readable<{
		[k: string]: AccountCfg;
	}>;
	showMyItemsOnly: Writable<boolean>;
	activeNetworkOrderbooks: Readable<Record<string, OrderbookCfg>>;
}
