<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import { twMerge } from 'tailwind-merge';
	import truncateEthAddress from 'truncate-eth-address';
	import type { AppKit } from '@reown/appkit';
	import type { Writable } from 'svelte/store';

	export let classes: string = '';

	export let appKitModal: Writable<AppKit>;
	export let connected: Writable<boolean>;
	export let signerAddress: Writable<string | null>;

	function handleClick() {
		$appKitModal.open();
	}
</script>

<Button
	data-testid="wallet-connect"
	on:click={handleClick}
	size="lg"
	class={twMerge('flex border border-gray-700 px-2 md:px-4 dark:border-gray-200', classes)}
	color={$connected ? 'alternative' : 'primary'}
>
	{#if $connected && $signerAddress}
		<span class="flex flex-row items-center gap-2 text-sm"
			><CheckCircleOutline color="green" size="sm" />{truncateEthAddress($signerAddress)}</span
		>
	{:else}
		<span>Connect Wallet</span>
	{/if}
</Button>
