<script lang="ts" generics="T">
	import { isEmpty } from 'lodash';
	import { Alert } from 'flowbite-svelte';
	import DropdownActiveSubgraphs from './dropdown/DropdownActiveSubgraphs.svelte';
	import DropdownOrderStatus from './dropdown/DropdownOrderStatus.svelte';
	import InputOrderHash from './input/InputOrderHash.svelte';
	import CheckboxZeroBalanceVault from './CheckboxZeroBalanceVault.svelte';
	import type { Writable } from 'svelte/store';
	import type { ConfigSource } from '../typeshare/config';
	import CheckboxMyItemsOnly from '$lib/components/CheckboxMyItemsOnly.svelte';

	export let settings: Writable<ConfigSource | undefined>;
	export let hideZeroBalanceVaults: Writable<boolean>;
	export let showMyItemsOnly: Writable<boolean>;
	export let activeSubgraphs: Writable<Record<string, string>>;
	export let activeOrderStatus: Writable<boolean | undefined>;
	export let orderHash: Writable<string>;
	export let isVaultsPage: boolean;
	export let isOrdersPage: boolean;
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
		<div class="mt-4 w-full lg:w-auto">
			<CheckboxMyItemsOnly context={isVaultsPage ? 'vaults' : 'orders'} {showMyItemsOnly} />
		</div>
		{#if isVaultsPage}
			<div class="mt-4 w-full lg:w-auto">
				<CheckboxZeroBalanceVault {hideZeroBalanceVaults} />
			</div>
		{/if}

		{#if isOrdersPage}
			<InputOrderHash {orderHash} />
			<DropdownOrderStatus {activeOrderStatus} />
		{/if}
		<DropdownActiveSubgraphs settings={$settings} {activeSubgraphs} />
	{/if}
</div>
