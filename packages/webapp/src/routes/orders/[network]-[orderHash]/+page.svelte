<script lang="ts">
	import {
		invalidateTanstackQueries,
		OrderDetail,
		PageHeader,
		useAccount,
		useToasts
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import { handleTransactionConfirmationModal } from '$lib/services/modal';
	import type { SgOrder, SgVault } from '@rainlanguage/orderbook';
	import { handleVaultAction } from '$lib/services/vaultActions';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
	import { handleRemoveOrder } from '$lib/services/handleRemoveOrder';
	import { useQueryClient } from '@tanstack/svelte-query';

	const queryClient = useQueryClient();
	const { orderHash, network } = $page.params;
	const { settings } = $page.data.stores;
	const orderbookAddress = $settings?.orderbooks[network]?.address;
	const subgraphUrl = $settings.subgraphs[network];
	const rpcUrl = $settings.networks[network]?.rpc;
	const chainId = $settings.networks[network]?.['chain-id'];
	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	async function onRemove(order: SgOrder) {
		await handleRemoveOrder(order, {
			handleTransactionConfirmationModal,
			errToast,
			manager,
			network,
			orderbookAddress: orderbookAddress as Hex,
			subgraphUrl,
			chainId,
			orderHash
		});
	}

	function onDeposit(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'deposit',
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex,
			queryClient,
			vaultId: orderHash
		});
	}

	function onWithdraw(vault: SgVault) {
		handleVaultAction({
			vault,
			action: 'withdraw',
			chainId,
			rpcUrl,
			subgraphUrl,
			account: $account as Hex,
			queryClient,
			vaultId: orderHash
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
