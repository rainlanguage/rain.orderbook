<script lang="ts">
	import {
		invalidateTanstackQueries,
		OrderDetail,
		PageHeader,
		useAccount
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleOrderRemoveModal } from '$lib/services/modal';
	import { useQueryClient } from '@tanstack/svelte-query';
	import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { handleVaultAction } from '$lib/services/handleVaultAction';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const { account } = useAccount();

	function onRemove(order: SgOrder) {
		handleOrderRemoveModal({
			open: true,
			args: {
				order,
				chainId,
				orderbookAddress,
				subgraphUrl,
				onRemove: () => {
					invalidateTanstackQueries(queryClient, [orderHash]);
				}
			}
		});
	}

	function onDeposit(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'deposit',
			queryClient,
			queryKey: $page.params.id,
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex
		});
	}

	function onWithdraw(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'withdraw',
			queryClient,
			queryKey: $page.params.id,
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex
		});
	}
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

<OrderDetail
	{orderHash}
	{subgraphUrl}
	{rpcUrl}
	{codeMirrorTheme}
	{lightweightChartsTheme}
	{colorTheme}
	{orderbookAddress}
	{onRemove}
	{onDeposit}
	{onWithdraw}
/>
