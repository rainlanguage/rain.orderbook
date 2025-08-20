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
	const { filterStore, currentVaultsFilters, isLoaded } = useFilterStore();
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

	// Track if we've completed the initial sync from FilterStore
	let hasCompletedInitialSync = false;
	const state = derived(
		[showMyItemsOnly, hideZeroBalanceVaults, selectedChainIds, activeTokens, account],
		([
			showMyItemsOnly,
			hideZeroBalanceVaults,
			selectedChainIds,
			activeTokens,
			accountVal
		]): GetVaultsFilters => {
			// Don't derive state until after initial sync to prevent overwriting URL params
			if (!hasCompletedInitialSync) {
				return {
					owners: [],
					hideZeroBalance: false,
					chainIds: undefined,
					tokens: undefined
				};
			}

			return {
				owners: showMyItemsOnly && accountVal ? [accountVal] : [],
				hideZeroBalance: hideZeroBalanceVaults,
				chainIds: selectedChainIds.length > 0 ? selectedChainIds : undefined,
				tokens: activeTokens.length > 0 ? activeTokens : undefined
			};
		}
	);

	// Sync from FilterStore to individual stores ONLY when first loaded
	// This ensures URL params are preserved and only loaded once
	$: {
		if ($isLoaded && !hasCompletedInitialSync) {
			const filters = $currentVaultsFilters;

			// Set UI stores based on loaded filters (FROM FilterStore TO UI)
			showMyItemsOnly.set(!!(filters.owners && filters.owners.length > 0));
			hideZeroBalanceVaults.set(!!filters.hideZeroBalance);
			selectedChainIds.set(filters.chainIds ?? []);
			activeTokens.set(filters.tokens ?? []);

			hasCompletedInitialSync = true;
		}
	}

	// Sync from individual stores to FilterStore
	// only after initial load to prevent overwriting persistent stores
	let isUpdating = false;
	const unsub = state.subscribe((filters) => {
		if (isUpdating || !hasCompletedInitialSync) return; // Wait for initial sync to complete
		isUpdating = true;
		$filterStore?.updateVaults((builder) =>
			builder
				.setOwners(filters.owners)
				.setHideZeroBalance(filters.hideZeroBalance)
				.setChainIds(filters.chainIds)
				.setTokens(filters.tokens)
		);
		setTimeout(() => (isUpdating = false), 0);
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
