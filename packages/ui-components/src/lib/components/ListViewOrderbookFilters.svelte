<script lang="ts" generics="T">
	import { isEmpty } from 'lodash';
	import { Alert, Tooltip } from 'flowbite-svelte';
	import DropdownActiveSubgraphs from './dropdown/DropdownActiveSubgraphs.svelte';
	import DropdownOrderStatus from './dropdown/DropdownOrderStatus.svelte';
	import DropdownOrderListAccounts from './dropdown/DropdownOrderListAccounts.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import type { Readable, Writable } from 'svelte/store';
	import type { ConfigSource } from '@rainlanguage/orderbook';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';
	export let settings: Writable<ConfigSource | undefined>;
	export let accounts: Readable<Record<string, string>> | undefined;
	export let hideZeroBalanceVaults: Writable<boolean>;
	export let activeAccountsItems: Writable<Record<string, string>> | undefined;
	export let showMyItemsOnly: Writable<boolean>;
	export let activeSubgraphs: Writable<Record<string, string>>;
	export let activeOrderStatus: Writable<boolean | undefined>;
	export let orderHash: Writable<string>;
	export let isVaultsPage: boolean;
	export let isOrdersPage: boolean;
	export let signerAddress: Writable<string | null> | undefined;
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
				<CheckboxMyItemsOnly
					context={isVaultsPage ? 'vaults' : 'orders'}
					{showMyItemsOnly}
					{signerAddress}
				/>
				{#if !$signerAddress}
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
			<DropdownOrderStatus {activeOrderStatus} />
		{/if}
		{#if $accounts && Object.values($accounts).length > 0}
			<DropdownOrderListAccounts {accounts} {activeAccountsItems} />
		{/if}
		<DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
	{/if}
</div>
