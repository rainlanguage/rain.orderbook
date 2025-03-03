<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import type { Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import { twMerge } from 'tailwind-merge';
	import truncateEthAddress from 'truncate-eth-address';
	export let signerAddress: Writable<string | null>;
	export let appKitModal: Writable<AppKit>;
	export let connected: Writable<boolean>;
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
	color={$connected ? 'alternative' : 'primary'}
>
	{#if $connected && $signerAddress}
		<div class="flex flex-col gap-1">
			<span class="flex flex-row items-center gap-2"
				><CheckCircleOutline color="green" size="md" />{truncateEthAddress($signerAddress)}</span
			>
		</div>
	{:else}
		<span>Connect Wallet</span>
	{/if}
</Button>
