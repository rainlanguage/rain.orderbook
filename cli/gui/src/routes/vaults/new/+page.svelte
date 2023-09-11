<script lang="ts">
	import Deposit from '../Deposit.svelte';
	import { FloatingLabelInput, Heading, P } from 'flowbite-svelte';
	import { fetchToken, generateVaultId, type Token } from '$lib';
	import Property from '$lib/components/Property.svelte';

	let token: Token;
	let vaultId: bigint = generateVaultId();

	let tokenAddress: string;
	let depositAmount: number;

	$: if (tokenAddress) {
		fetchToken(tokenAddress).then((r) => {
			if (r.isValid) token = r.token;
		});
	}
</script>

<div class="flex flex-col gap-y-4">
	<div class="flex flex-col gap-y-4 p-4 bg-white rounded-md border-gray-300 border">
		<Heading tag="h4">Create vault</Heading>
		<P>Deposit tokens to a new vault.</P>
		<FloatingLabelInput bind:value={tokenAddress} style="outlined" label="Token address" />
		{#if token}
			<Property label="Token name">{token.name}</Property>
			<Property label="Token symbol">{token.symbol}</Property>
		{/if}
	</div>

	{#if token && vaultId}
		<Deposit {token} {vaultId} />
	{/if}
</div>
