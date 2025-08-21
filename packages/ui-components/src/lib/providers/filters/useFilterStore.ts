import { getContext } from 'svelte';
import { get } from 'svelte/store';
import { FILTER_STORE_CONTEXT, type FilterStoreContext } from './FilterStoreProvider.svelte';
import { useAccount } from '$lib/providers/wallet/useAccount';
import type { Address } from '@rainlanguage/orderbook';

/**
 * Hook for accessing the filter store from any Svelte component.
 * Must be used within a component that is wrapped by FilterStoreProvider.
 *
 * @returns A FilterStoreContext containing the filter store, current vaults filters, current orders filters, and filter handlers.
 * @throws Error if called outside of FilterStoreProvider context
 *
 * @example
 * ```svelte
 * <script>
 *   import { useFilterStore } from '@rainlanguage/ui-components';
 *
 *   const { filterStore, currentVaultsFilters, currentOrdersFilters, ordersHandlers, vaultsHandlers } = useFilterStore();
 *
 *   // Use handlers for component callbacks
 *   const handleMyItemsChange = ordersHandlers.handleMyItemsOnlyChange;
 *
 *   $: console.log('Current vault filters', $currentVaultsFilters);
 *   $: console.log('Current orders filters', $currentOrdersFilters);
 * </script>
 * ```
 */
export function useFilterStore(): FilterStoreContext & {
	ordersHandlers: ReturnType<typeof createOrdersHandlers>;
	vaultsHandlers: ReturnType<typeof createVaultsHandlers>;
} {
	const store = getContext<FilterStoreContext>(FILTER_STORE_CONTEXT);

	if (!store) {
		throw new Error(
			'useFilterStore() must be called within a component wrapped by <FilterStoreProvider>'
		);
	}

	const { account } = useAccount();

	const ordersHandlers = createOrdersHandlers(store.filterStore, account);
	const vaultsHandlers = createVaultsHandlers(store.filterStore, account);

	return {
		...store,
		ordersHandlers,
		vaultsHandlers
	};
}

/**
 * Creates order filter handlers
 */
function createOrdersHandlers(
	filterStore: FilterStoreContext['filterStore'],
	account: ReturnType<typeof useAccount>['account']
) {
	return {
		handleMyItemsOnlyChange: (checked: boolean) => {
			get(filterStore)?.updateOrders((builder) => {
				if (checked && get(account)) {
					return builder.setOwners([get(account)!]);
				} else {
					return builder.setOwners([]);
				}
			});
		},

		handleActiveOrdersChange: (checked: boolean) => {
			get(filterStore)?.updateOrders((builder) => builder.setActive(checked ? undefined : true));
		},

		handleChainIdsChange: (chainIds: number[]) => {
			get(filterStore)?.updateOrders((builder) =>
				builder.setChainIds(chainIds.length > 0 ? chainIds : undefined)
			);
		},

		handleTokensChange: (tokens: Address[]) => {
			get(filterStore)?.updateOrders((builder) =>
				builder.setTokens(tokens.length > 0 ? tokens : undefined)
			);
		},

		handleOrderHashChange: (hash: string) => {
			get(filterStore)?.updateOrders((builder) =>
				builder.setOrderHash(
					hash && hash.length > 0 && hash !== '0x' ? (hash as `0x${string}`) : undefined
				)
			);
		},

		handleAccountsChange: (accountsRecord: Record<string, string>) => {
			const owners = Object.values(accountsRecord) as Address[];
			get(filterStore)?.updateOrders((builder) =>
				builder.setOwners(owners.length > 0 ? owners : [])
			);
		}
	};
}

/**
 * Creates vault filter handlers
 */
function createVaultsHandlers(
	filterStore: FilterStoreContext['filterStore'],
	account: ReturnType<typeof useAccount>['account']
) {
	return {
		handleMyItemsOnlyChange: (checked: boolean) => {
			get(filterStore)?.updateVaults((builder) => {
				if (checked && get(account)) {
					return builder.setOwners([get(account)!]);
				} else {
					return builder.setOwners([]);
				}
			});
		},

		handleZeroBalanceChange: (checked: boolean) => {
			get(filterStore)?.updateVaults((builder) => builder.setHideZeroBalance(checked));
		},

		handleChainIdsChange: (chainIds: number[]) => {
			get(filterStore)?.updateVaults((builder) =>
				builder.setChainIds(chainIds.length > 0 ? chainIds : undefined)
			);
		},

		handleTokensChange: (tokens: Address[]) => {
			get(filterStore)?.updateVaults((builder) =>
				builder.setTokens(tokens.length > 0 ? tokens : undefined)
			);
		}
	};
}
