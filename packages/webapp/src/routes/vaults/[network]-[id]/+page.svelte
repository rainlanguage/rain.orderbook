<script lang="ts">
	import {
		VaultBalanceChangesTable,
		VaultBalanceChart,
		QKEY_VAULT,
		TanstackPageContentDetail,
		bigintStringToHex,
		CardProperty,
		Hash,
		HashType
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { createQuery } from '@tanstack/svelte-query';
	import { getVault } from '@rainlanguage/orderbook/js_api';
	import { goto } from '$app/navigation';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
	import { formatUnits } from 'viem';

	const { settings } = $page.data.stores;
	const { lightweightChartsTheme } = $page.data.stores;
	const id = $page.params.id;
	const network = $page.params.network;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';

	$: vaultDetailQuery = createQuery({
		queryKey: [id, QKEY_VAULT + id],
		queryFn: () => {
			return getVault(subgraphUrl || '', id);
		},
		enabled: !!subgraphUrl
	});
</script>

<TanstackPageContentDetail query={vaultDetailQuery} emptyMessage="Vault not found">
	<svelte:fragment slot="top" let:data>
		<div
			data-testid="vaultDetailTokenName"
			class="flex gap-x-4 text-3xl font-medium dark:text-white"
		>
			{data.token.name}
		</div>
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<CardProperty data-testid="vaultDetailVaultId">
			<svelte:fragment slot="key">Vault ID</svelte:fragment>
			<svelte:fragment slot="value">{bigintStringToHex(data.vaultId)}</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailOrderbookAddress">
			<svelte:fragment slot="key">Orderbook</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash type={HashType.Identifier} value={data.orderbook.id} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailOwnerAddress">
			<svelte:fragment slot="key">Owner Address</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash type={HashType.Wallet} value={data.owner} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailTokenAddress">
			<svelte:fragment slot="key">Token address</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash value={data.token.id} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailBalance">
			<svelte:fragment slot="key">Balance</svelte:fragment>
			<svelte:fragment slot="value"
				>{formatUnits(BigInt(data.balance), Number(data.token.decimals ?? 0))}
				{data.token.symbol}</svelte:fragment
			>
		</CardProperty>

		<CardProperty>
			<svelte:fragment slot="key">Orders as input</svelte:fragment>
			<svelte:fragment slot="value">
				<p data-testid="vaultDetailOrdersAsInput" class="flex flex-wrap justify-start">
					{#if data.ordersAsInput && data.ordersAsInput.length > 0}
						{#each data.ordersAsInput as order}
							<Button
								class={'mr-1 mt-1 px-1 py-0' + (!order.active ? ' opacity-50' : '')}
								color={order.active ? 'green' : 'yellow'}
								data-order={order.id}
								data-testid={'vaultDetailOrderAsInputOrder' + order.id}
								on:click={() => goto(`/orders/${order.id}`)}
							>
								<Hash type={HashType.Identifier} value={order.orderHash} copyOnClick={false} />
							</Button>
						{/each}
					{:else}
						None
					{/if}
				</p>
			</svelte:fragment>
		</CardProperty>

		<CardProperty>
			<svelte:fragment slot="key">Orders as output</svelte:fragment>
			<svelte:fragment slot="value">
				<p data-testid="vaulDetailOrdersAsOutput" class="flex flex-wrap justify-start">
					{#if data.ordersAsOutput && data.ordersAsOutput.length > 0}
						{#each data.ordersAsOutput as order}
							<Button
								class={'mr-1 mt-1 px-1 py-0' + (!order.active ? ' opacity-50' : '')}
								color={order.active ? 'green' : 'yellow'}
								data-order={order.id}
								data-testid={'vaultDetailOrderAsOutputOrder' + order.id}
								on:click={() => goto(`/orders/${order.id}`)}
							>
								<Hash type={HashType.Identifier} value={order.orderHash} copyOnClick={false} />
							</Button>
						{/each}
					{:else}
						None
					{/if}
				</p>
			</svelte:fragment>
		</CardProperty>
	</svelte:fragment>

	<svelte:fragment slot="chart" let:data>
		<VaultBalanceChart vault={data} {subgraphUrl} {lightweightChartsTheme} />
	</svelte:fragment>

	<svelte:fragment slot="below"><VaultBalanceChangesTable {id} {subgraphUrl} /></svelte:fragment>
</TanstackPageContentDetail>
