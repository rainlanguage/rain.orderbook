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
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { useFilterStore } from '$lib/providers/filters';

	export let tokensQuery: Readable<QueryObserverResult<RaindexVaultToken[], Error>>;

	const { account } = useAccount();
	const { currentVaultsFilters, vaultsHandlers } = useFilterStore();
	const raindexClient = useRaindexClient();

	$: networks = raindexClient.getAllNetworks();
	$: accounts = raindexClient.getAllAccounts();

	// Direct access to current filter values for component props
	$: currentFilters = $currentVaultsFilters;
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
				<CheckboxMyItemsOnly
					context="vaults"
					currentOwners={currentFilters.owners}
					onChange={vaultsHandlers.handleMyItemsOnlyChange}
				/>
				{#if !$account}
					<Tooltip>Connect a wallet to filter by vault owner</Tooltip>
				{/if}
			</div>
		{/if}
		<div class="mt-4 w-full lg:w-auto">
			<CheckboxZeroBalanceVault
				checked={!!currentFilters.hideZeroBalance}
				onChange={vaultsHandlers.handleZeroBalanceChange}
			/>
		</div>
		<DropdownTokensFilter
			{tokensQuery}
			selectedTokens={currentFilters.tokens ?? []}
			onChange={vaultsHandlers.handleTokensChange}
			label="Tokens"
		/>
		<DropdownActiveNetworks
			selectedChainIds={currentFilters.chainIds ?? []}
			onChange={vaultsHandlers.handleChainIdsChange}
		/>
	{/if}
</div>
