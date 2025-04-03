<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';

	export let action: 'deposit' | 'withdraw';
	export let vault: SgVault;
	export let onDepositOrWithdraw: (vault: SgVault) => void;
	export let testId = `${action}-button`;
	export let disabled = false;
	export let label = '';

	function handleClick() {
		onDepositOrWithdraw(vault);
	}
</script>

<Button color="light" size="xs" data-testid={testId} {disabled} on:click={handleClick}>
	{#if action === 'deposit'}
		<ArrowDownOutline size="xs" />
	{:else if action === 'withdraw'}
		<ArrowUpOutline size="xs" />
	{/if}

	{#if label}
		{label}
	{/if}

	<slot></slot>
</Button>
