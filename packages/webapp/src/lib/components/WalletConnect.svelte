<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { appKitModal, connected } from '$lib/stores/wagmi';
	import { CheckCircleOutline } from 'flowbite-svelte-icons';

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
