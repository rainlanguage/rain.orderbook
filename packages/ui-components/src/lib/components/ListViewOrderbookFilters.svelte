<script lang="ts" generics="T">
	import { page } from '$app/stores';
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import Tooltip from './Tooltip.svelte';
	import DropdownActiveSubgraphs from './dropdown/DropdownActiveSubgraphs.svelte';
	import CheckboxActiveOrders from './checkbox/CheckboxActiveOrders.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import type { AppStoresInterface } from '$lib/types/appStores';

	export let settings: AppStoresInterface['settings'];
	export let accounts: AppStoresInterface['accounts'];
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];
	export let activeSubgraphs: AppStoresInterface['activeSubgraphs'];
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let orderHash: AppStoresInterface['orderHash'];

	$: isVaultsPage = $page.url.pathname === '/vaults';
	$: isOrdersPage = $page.url.pathname === '/orders';

	const { account } = useAccount();
</script>

<div
	class="grid w-full items-center gap-4 md:flex md:justify-end lg:min-w-[600px]"
	style="grid-template-columns: repeat(2, minmax(0, 1fr));"
>
	{#if isEmpty($settings?.networks)}
		<Alert color="gray" data-testid="no-networks-alert" class="w-full">
			No networks added to <a class="underline" href="/settings">settings</a>
		</Alert>
	{:else}
		{#if $accounts && !Object.values($accounts).length}
			<div class="mt-4 w-full lg:w-auto" data-testid="my-items-only">
				<CheckboxMyItemsOnly context={isVaultsPage ? 'vaults' : 'orders'} {showMyItemsOnly} />
				{#if !$account}
					<Tooltip>Connect a wallet to filter by {isVaultsPage ? 'vault' : 'order'} owner</Tooltip>
				{/if}
			</div>
		{/if}
		{#if isVaultsPage}
			<div class="mt-4 w-full lg:w-auto">
				<CheckboxZeroBalanceVault {hideZeroBalanceVaults} />
			</div>
		{/if}

		{#if isOrdersPage}
			<InputOrderHash {orderHash} />
			<div class="mt-4">
				<CheckboxActiveOrders {showInactiveOrders} />
			</div>
		{/if}
		{#if $accounts && Object.values($accounts).length > 0}
			<DropdownOrderListAccounts {accounts} {activeAccountsItems} />
		{/if}
		<DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
	{/if}
</div>
