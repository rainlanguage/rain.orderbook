<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
	import { createEventDispatcher } from 'svelte';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';

	const dispatch = createEventDispatcher<{
		click: {
			action: 'deposit' | 'withdraw';
			vault: SgVault;
			onSuccess?: () => void;
			[key: string]: any;
		};
	}>();

	export let action: 'deposit' | 'withdraw';
	export let vault: SgVault;
	export let onSuccess: (() => void) | undefined = undefined;
	export let additionalData: Record<string, any> = {};
	export let testId = `${action}-button`;
	export let disabled = false;
	export let label = '';

	function handleClick() {
		dispatch('click', {
			action,
			vault,
			onSuccess,
			...additionalData
		});
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
