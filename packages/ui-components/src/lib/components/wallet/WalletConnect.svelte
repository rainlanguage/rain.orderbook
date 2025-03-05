<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';
	import { twMerge } from 'tailwind-merge';
	import truncateEthAddress from 'truncate-eth-address';
	import { useSignerAddress, appKitModal } from '../../stores/wagmi';

	export let classes: string = '';

	console.log('use it!', useSignerAddress);

	const { signerAddress, connected } = useSignerAddress();
	console.log($signerAddress, $connected);

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
