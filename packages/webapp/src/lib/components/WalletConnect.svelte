<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { appKitModal, connected, signerAddress } from '$lib/stores/wagmi';

	function handleClick() {
		$appKitModal.open();
	}
</script>

{#if !$connected}
	<Button data-testid="wallet-connect" on:click={handleClick} class="w-full">Connect Wallet</Button>
{:else}
	<span data-testid="connected">Connected: {$signerAddress}</span>
{/if}
