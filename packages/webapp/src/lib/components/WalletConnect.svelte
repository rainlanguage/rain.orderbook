<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { appKitModal, connected, signerAddress } from '$lib/stores/wagmi';

	function handleClick() {
		$appKitModal.open();
	}
</script>

{#if !$connected}
	<Button on:click={handleClick} class="w-full">Connect Wallet</Button>
{:else}
	<span>Connected: {$signerAddress}</span>
{/if}
