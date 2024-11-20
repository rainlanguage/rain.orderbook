<script lang="ts">
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import {
		DropdownActiveSubgraphs,
		DropdownOrderStatus,
		DropdownOrderListAccounts,
		InputOrderHash,
		CheckboxZeroBalanceVault
	} from '@rainlanguage/ui-components';
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

<div class="flex min-w-[600px] items-center justify-end gap-x-2">
	{#if isEmpty($settings?.networks)}
		<Alert color="gray">
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
