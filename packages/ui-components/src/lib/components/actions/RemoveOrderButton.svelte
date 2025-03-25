<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import type { SgOrder } from '@rainlanguage/orderbook/js_api';

	const dispatch = createEventDispatcher<{
		click: {
			action: 'remove';
			order: SgOrder;
			onSuccess?: () => void;
		};
	}>();

	export let order: SgOrder;
	export let onSuccess: (() => void) | undefined = undefined;
	export let testId = 'remove-order-button';
	export let disabled = false;
	export let label = 'Remove';
	export let customClass: string = '';

	function handleClick() {
		dispatch('click', {
			action: 'remove',
			order,
			onSuccess
		});
	}
</script>

<Button class={customClass} data-testid={testId} {disabled} on:click={handleClick}>
	{#if label}
		{label}
	{/if}
	<slot></slot>
</Button>
