<script lang="ts">
	import { useWagmiClient } from '$lib/providers/wagmi/useWagmiClient';
	import { Button } from 'flowbite-svelte';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import { twMerge } from 'tailwind-merge';
	import truncateEthAddress from 'truncate-eth-address';
	const wagmiClient = useWagmiClient();
	const { signerAddress, connected, appKitModal } = wagmiClient;

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
		<span class="flex flex-row items-center gap-2 text-sm"
			><CheckCircleOutline class="outline-none" color="green" size="sm" />{truncateEthAddress(
				$signerAddress
			)}</span
		>
	{:else}
		<span>Connect Wallet</span>
	{/if}
</Button>
