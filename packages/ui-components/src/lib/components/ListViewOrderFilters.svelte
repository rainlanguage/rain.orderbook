<script lang="ts">
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import type { QueryObserverResult } from '@tanstack/svelte-query';
	import type { Readable } from 'svelte/store';
	import { derived, writable } from 'svelte/store';
	import DropdownActiveNetworks from './dropdown/DropdownActiveNetworks.svelte';
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import type { GetOrdersFilters, RaindexVaultToken, Address } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import Tooltip from './Tooltip.svelte';
	import DropdownTokensFilter from './dropdown/DropdownTokensFilter.svelte';
	import CheckboxActiveOrders from './checkbox/CheckboxActiveOrders.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { onDestroy } from 'svelte';
	import { useFilterStore } from '$lib/providers/filters';

	export let tokensQuery: Readable<QueryObserverResult<RaindexVaultToken[], Error>>;

	const { account } = useAccount();
	const { filterStore, currentOrdersFilters } = useFilterStore();
	const raindexClient = useRaindexClient();

	$: networks = raindexClient.getAllNetworks();
	$: accounts = raindexClient.getAllAccounts();

	//
	// Using Writable stores as mediator between components and filter store
	// This follows the same pattern as ListViewVaultFilters.svelte
	//
	const showMyItemsOnly = writable(false);
	const showInactiveOrders = writable(false);
	const selectedChainIds = writable<number[]>([]);
	const activeTokens = writable<Address[]>([]);
	const orderHash = writable<Hex>(undefined);
	const activeAccountsItems = writable<Record<string, Address>>({});

	const state = derived(
		[
			showMyItemsOnly,
			showInactiveOrders,
			selectedChainIds,
			activeTokens,
			orderHash,
			activeAccountsItems,
			account
		],
		([
			showMyItemsOnly,
			showInactiveOrders,
			selectedChainIds,
			activeTokens,
			orderHash,
			activeAccountsItems,
			accountVal
		]): GetOrdersFilters => {
			// Determine owners: prioritize activeAccountsItems, then showMyItemsOnly
			let owners: Address[] = [];
			if (Object.keys(activeAccountsItems).length > 0) {
				owners = Object.values(activeAccountsItems);
			} else if (showMyItemsOnly && accountVal) {
				owners = [accountVal];
			}

			return {
				owners,
				active: showInactiveOrders ? undefined : true, // undefined means show all, true means only active
				orderHash: orderHash ? (orderHash as Hex) : undefined,
				tokens: activeTokens.length > 0 ? activeTokens : undefined,
				chainIds: selectedChainIds.length > 0 ? selectedChainIds : undefined
			};
		}
	);

	// Sync from FilterStore to individual stores
	// to preload actual filter values in UI components
	$: {
		if ($currentOrdersFilters) {
			const filters = $currentOrdersFilters;
			showMyItemsOnly.set(
				!!(
					filters.owners &&
					filters.owners.length > 0 &&
					Object.keys($activeAccountsItems).length === 0
				)
			);
			showInactiveOrders.set(filters.active === undefined);
			selectedChainIds.set(filters.chainIds ?? []);
			activeTokens.set(filters.tokens ?? []);
			orderHash.set(filters.orderHash as Hex);
			// Note: activeAccountsItems is handled separately by DropdownOrderListAccounts
		}
	}

	// Sync from individual stores to FilterStore (with protection against update loops)
	let isUpdating = false;
	const unsub = state.subscribe((filters) => {
		if (isUpdating) return;
		isUpdating = true;
		$filterStore?.updateOrders((builder) =>
			builder
				.setOwners(filters.owners)
				.setActive(filters.active)
				.setOrderHash(filters.orderHash)
				.setTokens(filters.tokens)
				.setChainIds(filters.chainIds)
		);
		setTimeout(() => (isUpdating = false), 0);
	});

	onDestroy(() => unsub());

	$: selectedTokens =
		$activeTokens?.filter(
			(address) => !$tokensQuery.data || $tokensQuery.data.some((t) => t.address === address)
		) ?? [];
</script>

<div
	class="grid w-full items-center gap-4 md:flex md:justify-end lg:min-w-[600px]"
	style="grid-template-columns: repeat(2, minmax(0, 1fr));"
>
	{#if networks.error || isEmpty(networks.value)}
		<Alert color="gray" data-testid="no-networks-alert" class="w-full">
			No networks added to <a class="underline" href="/settings">settings</a>
		</Alert>
	{:else}
		{#if !accounts.error}
			{#if accounts.value.size === 0}
				<div class="mt-4 w-full lg:w-auto" data-testid="my-items-only">
					<CheckboxMyItemsOnly context="orders" {showMyItemsOnly} />
					{#if !$account}
						<Tooltip>Connect a wallet to filter by order owner</Tooltip>
					{/if}
				</div>
			{:else}
				<DropdownOrderListAccounts {activeAccountsItems} />
			{/if}
		{/if}

		<InputOrderHash {orderHash} />

		<div class="mt-4">
			<CheckboxActiveOrders {showInactiveOrders} />
		</div>

		<DropdownTokensFilter {tokensQuery} {activeTokens} {selectedTokens} label="Tokens" />

		<DropdownActiveNetworks {selectedChainIds} />
	{/if}
</div>
