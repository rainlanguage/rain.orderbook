<script lang="ts">
	import { network } from 'svelte-wagmi-stores';
	import type { TokenVaultsQuery } from '$lib/gql/generated/graphql';
	import { orderbook } from '$lib';
	import { Alert, Button, FloatingLabelInput, Heading, Spinner } from 'flowbite-svelte';

	export let vault: TokenVaultsQuery['tokenVaults'][0];

	let amount: number;
	$: withdrawAmount = amount
		? BigInt(amount) * BigInt(10) ** BigInt(vault.token.decimals)
		: BigInt(0);

	$: ({ write, error, isLoading, isSuccess, isError, data } = $orderbook.write({
		functionName: 'withdraw',
		args: [
			{
				vaultId: vault.vaultId,
				token: vault.token.id as `0x${string}`,
				amount: withdrawAmount
			}
		],
		onSuccess: ({ receipt }) => {
			console.log(receipt);
		}
	}));
</script>

<div class="gap-y-4 flex flex-col p-4 border border-gray-300 rounded-lg items-start">
	<Heading tag="h6">Withdraw</Heading>
	<FloatingLabelInput label="Amount" style="outlined" type="number" bind:value={amount} />
	<Button
		disabled={$isLoading || withdrawAmount == BigInt(0) || vault.balance < withdrawAmount}
		class="whitespace-nowrap"
		on:click={write}
	>
		{#if $isLoading}
			<Spinner size="5" class="mr-2" />
			<span>Confirming...</span>
		{:else}
			<span>Withdraw</span>
		{/if}
	</Button>

	{#if $isSuccess || $isError}
		<Alert
			color={$isSuccess ? 'green' : 'red'}
			class="max-w-full whitespace-break-spaces overflow-clip w-full"
		>
			{#if $isSuccess}
				<p>Confirmed</p>
				<p>
					<a
						target="_blank"
						href={`${$network?.chain?.blockExplorers?.default.url}/tx/${$data?.hash}`}
						>Transaction: {$data?.hash}</a
					>
				</p>
			{:else if $isError}
				<p>{$error}</p>
			{/if}
		</Alert>
	{/if}
</div>
