<script lang="ts">
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import type { QueryObserverResult } from '@tanstack/svelte-query';
	import type { Readable } from 'svelte/store';
	import DropdownActiveNetworks from './dropdown/DropdownActiveNetworks.svelte';
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import type { RaindexVaultToken } from '@rainlanguage/orderbook';
	import Tooltip from './Tooltip.svelte';
	import DropdownTokensFilter from './dropdown/DropdownTokensFilter.svelte';
	import CheckboxActiveOrders from './checkbox/CheckboxActiveOrders.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { useFilterStore } from '$lib/providers/filters';

	export let tokensQuery: Readable<QueryObserverResult<RaindexVaultToken[], Error>>;

	const { account } = useAccount();
	const { currentOrdersFilters, ordersHandlers } = useFilterStore();
	const raindexClient = useRaindexClient();

	$: networks = raindexClient.getAllNetworks();
	$: accounts = raindexClient.getAllAccounts();

	// Direct access to current filter values for component props
	$: currentFilters = $currentOrdersFilters;

	// Convert owners to activeAccountsItems format
	$: activeAccountsItems = (() => {
		if (!currentFilters.owners || currentFilters.owners.length === 0) return {};
		if (accounts.error || !accounts.value) return {};

		const accountsMap = new Map();
		for (const [name, address] of accounts.value.entries()) {
			accountsMap.set(address, name);
		}

		const result: Record<string, string> = {};
		currentFilters.owners.forEach((owner) => {
			const name = accountsMap.get(owner);
			if (name) {
				result[name] = owner;
			}
		});
		return result;
	})();
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
					<CheckboxMyItemsOnly
						context="orders"
						currentOwners={currentFilters.owners}
						onChange={ordersHandlers.handleMyItemsOnlyChange}
					/>
					{#if !$account}
						<Tooltip>Connect a wallet to filter by order owner</Tooltip>
					{/if}
				</div>
			{:else}
				<DropdownOrderListAccounts
					{activeAccountsItems}
					onChange={ordersHandlers.handleAccountsChange}
				/>
			{/if}
		{/if}

		<InputOrderHash
			value={currentFilters.orderHash ?? ''}
			onChange={ordersHandlers.handleOrderHashChange}
		/>

		<div class="mt-4">
			<CheckboxActiveOrders
				checked={currentFilters.active === undefined}
				onChange={ordersHandlers.handleActiveOrdersChange}
			/>
		</div>

		<DropdownTokensFilter
			{tokensQuery}
			selectedTokens={currentFilters.tokens ?? []}
			onChange={ordersHandlers.handleTokensChange}
			label="Tokens"
		/>

		<DropdownActiveNetworks
			selectedChainIds={currentFilters.chainIds ?? []}
			onChange={ordersHandlers.handleChainIdsChange}
		/>
	{/if}
</div>
