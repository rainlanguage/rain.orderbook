<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import type { Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	export let appKitModal: Writable<AppKit>;
	export let connected: Writable<boolean>;
	function handleClick() {
		$appKitModal.open();
	}
</script>

<Button
	data-testid="wallet-connect"
	on:click={handleClick}
	class="flex w-full border border-gray-700 dark:border-gray-200"
	color={$connected ? 'alternative' : 'primary'}
>
	{#if $connected}
		<span class="flex flex-row items-center gap-2"
			><CheckCircleOutline color="green" size="md" />Connected</span
		>
	{:else}
		<span>Connect Wallet</span>
	{/if}
</Button>
