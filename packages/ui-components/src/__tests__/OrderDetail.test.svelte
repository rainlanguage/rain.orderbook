<script lang="ts">
	import TanstackPageContentDetail from '../lib/components/detail/TanstackPageContentDetail.svelte';
	import CardProperty from '../lib/components/CardProperty.svelte';
	import ButtonVaultLink from '../lib/components/ButtonVaultLink.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import type { OrderWithSortedVaults, SgVault } from '@rainlanguage/orderbook/js_api';
	import { getOrderByHash } from '@rainlanguage/orderbook/js_api';
	import { QKEY_ORDER } from '../lib/queries/keys';
	import { Button } from 'flowbite-svelte';
	import Refresh from '$lib/components/icon/Refresh.svelte';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { OrderRemoveModalProps } from '../lib/types/modal';
	import { isAddress, isAddressEqual, type Hex } from 'viem';
	import { invalidateIdQuery } from '$lib/queries/queryClient';
	import VaultActionButton from '$lib/components/actions/VaultActionButton.svelte';
	import { createEventDispatcher } from 'svelte';
	import type { Writable } from 'svelte/store';

	const queryClient = useQueryClient();

	const dispatch = createEventDispatcher<{
		deposit: { vault: SgVault };
		withdraw: { vault: SgVault };
	}>();

	export let handleOrderRemoveModal: ((props: OrderRemoveModalProps) => void) | undefined =
		undefined;
	export let orderHash: string;
	export let subgraphUrl: string;
	export let chainId: number;
	export let orderbookAddress: Hex;
	export let signerAddress: Writable<Hex | null> | undefined = undefined;
	export let onDeposit: (vault: SgVault) => void;
	export let onWithdraw: (vault: SgVault) => void;

	$: orderDetailQuery = createQuery<OrderWithSortedVaults>({
		queryKey: [orderHash, QKEY_ORDER + orderHash],
		queryFn: () => {
			return getOrderByHash(subgraphUrl, orderHash);
		},
		enabled: !!subgraphUrl
	});
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
	<svelte:fragment slot="top" let:data>
		<div>Order {data.order.orderHash}</div>
		{#if data && $signerAddress && isAddress($signerAddress) && isAddress(data.order.owner) && isAddressEqual($signerAddress, data.order.owner) && data.order.active && handleOrderRemoveModal}
			<Button
				data-testid="remove-button"
				color="dark"
				on:click={() =>
					handleOrderRemoveModal({
						open: true,
						args: {
							order: data.order,
							onRemove: $orderDetailQuery.refetch,
							chainId,
							orderbookAddress,
							subgraphUrl
						}
					})}
				disabled={!handleOrderRemoveModal}
			>
				Remove
			</Button>
		{/if}

		<Refresh
			on:click={async () => await invalidateIdQuery(queryClient, orderHash)}
			spin={$orderDetailQuery.isLoading || $orderDetailQuery.isFetching}
		/>
	</svelte:fragment>

	<svelte:fragment slot="card" let:data>
		<div>Owner: {data.order.owner}</div>

		{#each [{ key: 'Input vaults', type: 'inputs' }, { key: 'Output vaults', type: 'outputs' }, { key: 'Input & output vaults', type: 'inputs_outputs' }] as { key, type }}
			{#if data.vaults.get(type)?.length !== 0}
				<CardProperty>
					<svelte:fragment slot="key">{key}</svelte:fragment>
					<svelte:fragment slot="value">
						<div class="mt-2 space-y-2">
							{#each data.vaults.get(type) || [] as vault}
								<ButtonVaultLink tokenVault={vault} subgraphName="subgraphName">
									<svelte:fragment slot="buttons">
										{#if $signerAddress && isAddress($signerAddress) && isAddress(vault.owner) && isAddressEqual($signerAddress, vault.owner) && chainId}
											<div class="flex gap-1">
												<VaultActionButton
													action="deposit"
													{vault}
													testId="deposit-button"
													onDepositOrWithdraw={onDeposit}
												/>
												<VaultActionButton
													action="withdraw"
													{vault}
													testId="withdraw-button"
													onDepositOrWithdraw={onWithdraw}
												/>
											</div>
										{/if}
									</svelte:fragment>
								</ButtonVaultLink>
							{/each}
						</div>
					</svelte:fragment>
				</CardProperty>
			{/if}
		{/each}
	</svelte:fragment>

	<svelte:fragment slot="chart">
		<div>Chart placeholder</div>
	</svelte:fragment>

	<svelte:fragment slot="below" let:data>
		<div>Below content: {data.order.orderHash}</div>
	</svelte:fragment>
</TanstackPageContentDetail>
