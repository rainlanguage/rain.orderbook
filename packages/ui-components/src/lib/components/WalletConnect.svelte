<script lang="ts">
	import { web3Modal, signerAddress, connected } from 'svelte-wagmi';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Button } from 'flowbite-svelte';
</script>

<Button on:click={() => $web3Modal.open()} tabindex={0} data-testid="wallet-connect">
	{#if !$signerAddress || !$connected}
		<div data-testid="not-connected">
			<span>Connect Wallet</span><span class="hidden md:inline"> </span>
		</div>
	{:else}
		<div data-testid="connected">
			<span>Connected</span>
		</div>
	{/if}
</Button>
