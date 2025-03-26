<script lang="ts">
	import TanstackPageContentDetail from '../lib/components/detail/TanstackPageContentDetail.svelte';
	import CardProperty from '../lib/components/CardProperty.svelte';
	import ButtonVaultLink from '../lib/components/ButtonVaultLink.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import type { OrderWithSortedVaults, SgOrder } from '@rainlanguage/orderbook/js_api';
	import { getOrderByHash } from '@rainlanguage/orderbook/js_api';
	import { QKEY_ORDER } from '../lib/queries/keys';
	import type { Readable, Writable } from 'svelte/store';
	import DepositOrWithdrawButtons from '../lib/components/detail/DepositOrWithdrawButtons.svelte';
	import Refresh from '$lib/components/icon/Refresh.svelte';
	import { useQueryClient } from '@tanstack/svelte-query';
	import { invalidateIdQuery } from '$lib/queries/queryClient';
	import { createEventDispatcher } from 'svelte';
	import RemoveOrderButton from '../lib/components/actions/RemoveOrderButton.svelte';

	const queryClient = useQueryClient();
	const dispatch = createEventDispatcher<{
		remove: { order: SgOrder };
	}>();

	export let walletAddressMatchesOrBlank: Readable<(address: string) => boolean> | undefined =
		undefined;
	export let orderHash: string;
	export let subgraphUrl: string;
	export let signerAddress: Writable<string>;
	export let chainId: number;

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
		{#if $signerAddress === data.order.owner || $walletAddressMatchesOrBlank?.(data.order.owner)}
			{#if data.order.active}
				<RemoveOrderButton
					order={data.order}
					onSuccess={() => $orderDetailQuery.refetch()}
					on:remove={(e) => dispatch('remove', { order: e.detail.order })}
				/>
			{/if}
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
										{#if $signerAddress === data.order.owner && chainId}
											<DepositOrWithdrawButtons
												{vault}
												chainId={1}
												rpcUrl="https://example.com"
												query={orderDetailQuery}
												handleDepositOrWithdrawModal={() => {}}
												{subgraphUrl}
											/>
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
