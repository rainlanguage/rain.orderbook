import type {
	AppStoresInterface,
	ConfigSource,
	OrderbookConfigSource,
	OrderbookCfgRef
} from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';
import pkg from 'lodash';

const { pickBy } = pkg;

export interface LayoutData {
	stores: AppStoresInterface;
}

export const load = async ({ fetch }) => {
	const response = await fetch(
		'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/settings.json'
	);
	const settingsJson = await response.json();
	const settings = writable<ConfigSource | undefined>(settingsJson);
	const activeNetworkRef = writable<string>('');
	const activeOrderbookRef = writable<string>('');
	const activeOrderbook = derived(
		[settings, activeOrderbookRef],
		([$settings, $activeOrderbookRef]) =>
			$settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined
				? $settings.orderbooks[$activeOrderbookRef]
				: undefined
	);

	const activeNetworkOrderbooks = derived(
		[settings, activeNetworkRef],
		([$settings, $activeNetworkRef]) =>
			$settings?.orderbooks
				? (pickBy(
						$settings.orderbooks,
						(orderbook) => orderbook.network === $activeNetworkRef
					) as Record<OrderbookCfgRef, OrderbookConfigSource>)
				: ({} as Record<OrderbookCfgRef, OrderbookConfigSource>)
	);

	const accounts = derived(settings, ($settings) => $settings?.accounts);
	const activeAccountsItems = writable<Record<string, string>>({});

	const subgraphUrl = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
		$settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
			? $settings.subgraphs[$activeOrderbook.subgraph]
			: undefined
	);
	const activeAccounts = derived(
		[accounts, activeAccountsItems],
		([$accounts, $activeAccountsItems]) =>
			Object.keys($activeAccountsItems).length === 0
				? {}
				: Object.fromEntries(
						Object.entries($accounts || {}).filter(([key]) => key in $activeAccountsItems)
					)
	);

	return {
		stores: {
			settings,
			activeSubgraphs: writable<Record<string, string>>({}),
			accounts,
			activeAccountsItems,
			activeAccounts,
			activeOrderStatus: writable<boolean | undefined>(undefined),
			orderHash: writable<string>(''),
			hideZeroBalanceVaults: writable<boolean>(false),
			activeNetworkRef,
			activeOrderbookRef,
			activeOrderbook,
			subgraphUrl,
			activeNetworkOrderbooks
		}
	};
};

export const ssr = false;
