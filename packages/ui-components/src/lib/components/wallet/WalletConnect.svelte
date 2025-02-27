<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import type { Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import { twMerge } from 'tailwind-merge';
	export let appKitModal: Writable<AppKit>;
	export let connected: boolean;
	export let classes: string = '';
	function handleClick() {
		$appKitModal.open();
	}
</script>

<Button
	data-testid="wallet-connect"
	on:click={handleClick}
	size="lg"
	class={twMerge('flex border border-gray-700 px-2 md:px-4 dark:border-gray-200', classes)}
	color={connected ? 'alternative' : 'primary'}
>
	{#if connected}
		<span class="flex flex-row items-center gap-2"
			><CheckCircleOutline color="green" size="md" />Connected</span
		>
	{:else}
		<span>Connect Wallet</span>
	{/if}
</Button>
