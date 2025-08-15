<script lang="ts">
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import type { QueryObserverResult } from '@tanstack/svelte-query';
	import type { Readable } from 'svelte/store';
	import { derived, writable } from 'svelte/store';
	import DropdownActiveNetworks from './dropdown/DropdownActiveNetworks.svelte';
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import type { GetVaultsFilters, RaindexVaultToken } from '@rainlanguage/orderbook';
	import Tooltip from './Tooltip.svelte';
	import DropdownTokensFilter from './dropdown/DropdownTokensFilter.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { onDestroy } from 'svelte';
	import { useFilterStore } from '$lib/providers/filters';

	export let tokensQuery: Readable<QueryObserverResult<RaindexVaultToken[], Error>>;

	const { account } = useAccount();
	const { filterStore, currentVaultsFilters } = useFilterStore();
	const raindexClient = useRaindexClient();

	$: networks = raindexClient.getAllNetworks();
	$: accounts = raindexClient.getAllAccounts();

	//
	// Since OrderFilters are still rely on the old filter logic,
	// to avoid duplication of logic of each child component,
	// we will keep using Writable stores as mediator between components
	// and actual filter store.
	//
	const showMyItemsOnly = writable(false);
	const hideZeroBalanceVaults = writable(false);
	const selectedChainIds = writable<number[]>([]);
	const activeTokens = writable<`0x${string}`[]>([]);

	const state = derived(
		[showMyItemsOnly, hideZeroBalanceVaults, selectedChainIds, activeTokens, account],
		([
			showMyItemsOnly,
			hideZeroBalanceVaults,
			selectedChainIds,
			activeTokens,
			accountVal
		]): GetVaultsFilters => ({
			owners: showMyItemsOnly && accountVal ? [accountVal] : [],
			hideZeroBalance: hideZeroBalanceVaults,
			chainIds: selectedChainIds.length > 0 ? selectedChainIds : undefined,
			tokens: activeTokens.length > 0 ? activeTokens : undefined
		})
	);

	// Sync from FilterStore to individual stores
	// to preload actual filter values in UI components
	let isInited = false;
	$: {
		if (!isInited && $currentVaultsFilters) {
			const filters = $currentVaultsFilters;
			showMyItemsOnly.set(!!(filters.owners && filters.owners.length > 0));
			hideZeroBalanceVaults.set(!!filters.hideZeroBalance);
			selectedChainIds.set(filters.chainIds ?? []);
			activeTokens.set(filters.tokens ?? []);
			isInited = true;
		}
	}

	// Sync from individual stores to FilterStore
	const unsub = state.subscribe((filters) => {
		if (!isInited) return;
		$filterStore?.updateVaults((builder) =>
			builder
				.setOwners(filters.owners)
				.setHideZeroBalance(filters.hideZeroBalance)
				.setChainIds(filters.chainIds)
				.setTokens(filters.tokens)
		);
	});

	onDestroy(() => unsub());

	// TODO: Once OrderFilters are fully migrated to the new filter store,
	// we can remove these legacy props and use the filter store directly
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
			<div class="mt-4 w-full lg:w-auto" data-testid="my-items-only">
				<CheckboxMyItemsOnly context="vaults" {showMyItemsOnly} />
				{#if !$account}
					<Tooltip>Connect a wallet to filter by vault owner</Tooltip>
				{/if}
			</div>
		{/if}
		<div class="mt-4 w-full lg:w-auto">
			<CheckboxZeroBalanceVault {hideZeroBalanceVaults} />
		</div>
		<DropdownTokensFilter
			{tokensQuery}
			{activeTokens}
			selectedTokens={$activeTokens}
			label="Tokens"
		/>
		<DropdownActiveNetworks {selectedChainIds} />
	{/if}
</div>
