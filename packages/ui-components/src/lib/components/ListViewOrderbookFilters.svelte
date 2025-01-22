<script lang="ts" generics="T">
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import DropdownActiveSubgraphs from './dropdown/DropdownActiveSubgraphs.svelte';
	import DropdownOrderStatus from './dropdown/DropdownOrderStatus.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import type { Readable, Writable } from 'svelte/store';
	import type { ConfigSource } from '../typeshare/config';

	export let settings: Writable<ConfigSource | undefined>;
	export let accounts: Readable<Record<string, string>>;
	export let hideZeroBalanceVaults: Writable<boolean>;
	export let activeAccountsItems: Writable<Record<string, string>>;
	export let activeSubgraphs: Writable<Record<string, string>>;
	export let activeOrderStatus: Writable<boolean | undefined>;
	export let orderHash: Writable<string>;
	export let isVaultsPage: boolean;
	export let isOrdersPage: boolean;
</script>

<div class="flex flex-wrap items-center justify-end gap-x-2 lg:min-w-[600px]">
	{#if isEmpty($settings?.networks)}
		<Alert color="gray" data-testid="no-networks-alert">
			No networks added to <a class="underline" href="/settings">settings</a>
		</Alert>
	{:else}
		{#if isVaultsPage}
			<div class="mt-4">
				<CheckboxZeroBalanceVault {hideZeroBalanceVaults} />
			</div>
		{/if}

		{#if isOrdersPage}
			<InputOrderHash {orderHash} />
			<DropdownOrderStatus {activeOrderStatus} />
		{/if}
		<DropdownOrderListAccounts {accounts} {activeAccountsItems} />
		<DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
	{/if}
</div>
