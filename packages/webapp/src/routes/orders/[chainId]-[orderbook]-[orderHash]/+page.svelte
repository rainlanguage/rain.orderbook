<script lang="ts">
	import { OrderDetail, PageHeader, useAccount, useToasts } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { codeMirrorTheme, lightweightChartsTheme, colorTheme } from '$lib/darkMode';
	import {
		handleDepositModal,
		handleTransactionConfirmationModal,
		handleWithdrawModal
	} from '$lib/services/modal';
	import type { Address, RaindexClient, RaindexOrder, RaindexVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { useTransactions } from '@rainlanguage/ui-components';
	import { handleRemoveOrder } from '$lib/services/handleRemoveOrder';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';
	import { handleVaultDeposit } from '$lib/services/handleVaultDeposit';

	const { orderHash, chainId, orderbook } = $page.params;
	const parsedOrderHash = orderHash as Hex;
	const parsedChainId = Number(chainId);
	const orderbookAddress = orderbook as Address;

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	async function onRemove(raindexClient: RaindexClient, order: RaindexOrder) {
		await handleRemoveOrder({
			raindexClient,
			order,
			handleTransactionConfirmationModal,
			errToast,
			manager
		});
	}

	async function onDeposit(raindexClient: RaindexClient, vault: RaindexVault) {
		await handleVaultDeposit({
			raindexClient,
			vault,
			handleDepositModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			account: $account as Hex
		});
	}

	async function onWithdraw(raindexClient: RaindexClient, vault: RaindexVault) {
		await handleVaultWithdraw({
			raindexClient,
			vault,
			handleWithdrawModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			account: $account as Hex
		});
	}
</script>

<PageHeader title="Order" pathname={$page.url.pathname} />

<OrderDetail
	chainId={parsedChainId}
	{orderbookAddress}
	orderHash={parsedOrderHash}
	{lightweightChartsTheme}
	{colorTheme}
	{codeMirrorTheme}
	{onRemove}
	{onDeposit}
	{onWithdraw}
/>
