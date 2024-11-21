<script lang="ts" generics="T">
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import Refresh from './icon/Refresh.svelte';
	import DropdownActiveSubgraphs from './dropdown/DropdownActiveSubgraphs.svelte';
	import DropdownOrderStatus from './dropdown/DropdownOrderStatus.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import type { Readable, Writable } from 'svelte/store';
	import type { ConfigSource } from '../typeshare/config';
	import type { CreateInfiniteQueryResult, InfiniteData } from '@tanstack/svelte-query';

	export let settings: Writable<ConfigSource | undefined>;
	export let accounts: Readable<Record<string, string>>;
	export let hideZeroBalanceVaults: Writable<boolean>;
	export let activeAccountsItems: Writable<Record<string, string>>;
	export let activeSubgraphs: Writable<Record<string, string>>;
	export let activeOrderStatus: Writable<boolean | undefined>;
	export let orderHash: Writable<string>;
	export let isVaultsPage: boolean;
	export let isOrdersPage: boolean;
	export let query: CreateInfiniteQueryResult<InfiniteData<unknown[], unknown>, Error>;
</script>

<div class="flex min-w-[600px] items-center justify-end gap-x-2">
	<Refresh
		data-testid="refresh-button"
		class="mr-2 h-6 w-6 cursor-pointer text-gray-400 dark:text-gray-400"
		spin={$query.isLoading || $query.isFetching}
		on:click={() => {
			console.log('REFETCHING!');
			$query.refetch();
		}}
	/>
	{#if isEmpty($settings?.networks)}
		<Alert color="gray" data-testid="no-networks-alert">
			No networks added to <a class="underline" href="/settings">settings</a>
		</Alert>
	{:else}
		{#if isVaultsPage}
			<CheckboxZeroBalanceVault {hideZeroBalanceVaults} />
		{/if}

		{#if isOrdersPage}
			<InputOrderHash {orderHash} />
			<DropdownOrderStatus {activeOrderStatus} />
		{/if}
		<DropdownOrderListAccounts {accounts} {activeAccountsItems} />
		<DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
	{/if}
</div>
