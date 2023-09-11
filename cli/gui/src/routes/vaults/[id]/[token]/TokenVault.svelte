<script lang="ts">
	import { hexToBigInt, toHex, getAddress } from 'viem';
	import { queries } from '$lib';
	import { Heading, Spinner } from 'flowbite-svelte';
	import Property from '$lib/components/Property.svelte';
	import { account } from 'svelte-wagmi-stores';
	import Withdraw from './Withdraw.svelte';
	import DepositExisting from './DepositExisting.svelte';

	export let vaultId: `0x${string}`;
	export let token: `0x${string}`;

	const { result } = queries.queryTokenVaults({
		vaultIds: [hexToBigInt(vaultId).toString()],
		tokens: [token]
	});
</script>

{#if $result?.data?.length}
	{@const vault = $result?.data[0]}
	<div class="flex flex-col gap-y-2">
		<div class="flex flex-col gap-y-4">
			<div class="gap-y-4 flex flex-col p-4 border border-gray-300 rounded-lg">
				<Property label="Vault id">{toHex(BigInt(vault.vaultId))}</Property>
				<Property label="Owner">{vault.owner.id}</Property>
				<Property label="Vault balance">{vault.balanceDisplay}</Property>
				<Property label="Token"
					><p>
						{vault.token.name} ({vault.token.symbol})
					</p>
					<p class="text-gray-500">
						{vault.token.id}
					</p>
				</Property>
				<Property label="Linked orders">
					{#if vault?.orders?.length}
						{#each vault.orders as order}
							<p>{order.id}</p>
						{/each}
					{:else}
						None
					{/if}
				</Property>
			</div>
		</div>
		{#if $account?.address && getAddress($account?.address) == getAddress(vault.owner.id)}
			<div class="flex flex-col gap-y-4">
				<Heading tag="h4">Actions</Heading>
				<Withdraw {vault} />
				<DepositExisting {vault} />
			</div>
		{/if}
	</div>
{:else if $result?.error || !$result?.data?.length}
	<p>{$result?.error?.message || 'Not found'}</p>
{:else}
	<Spinner />
{/if}
