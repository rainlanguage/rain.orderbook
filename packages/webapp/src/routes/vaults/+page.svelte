<script lang="ts">
	import {
		PageHeader,
		useAccount,
		useToasts,
		useTransactions,
		VaultsListTable
	} from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import {
		hideZeroBalanceVaults,
		showMyItemsOnly,
		orderHash,
		activeTokens
	} from '$lib/stores/settings';
	import { selectedChainIds } from '$lib/stores/settings';
	import { handleMultipleVaultsWithdraw } from '$lib/services/handleMultipleVaultsWithdraw';
	import type { RaindexClient, RaindexVault } from '@rainlanguage/orderbook';
	import {
		handleTransactionConfirmationModal,
		handleWithdrawMultipleModal
	} from '../../lib/services/modal';
	import type { Hex } from 'viem';

	const { settings, accounts, activeAccountsItems, showInactiveOrders } = $page.data.stores;

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	function onWithdrawMultiple(raindexClient: RaindexClient, vaults: RaindexVault[]) {
		return handleMultipleVaultsWithdraw({
			raindexClient,
			vaults,
			handleWithdrawModal: handleWithdrawMultipleModal,
			handleTransactionConfirmationModal,
			errToast,
			manager,
			account: $account as Hex
		});
	}
</script>

<PageHeader title="Vaults" pathname={$page.url.pathname} />

<VaultsListTable
	{orderHash}
	{showMyItemsOnly}
	{activeAccountsItems}
	{showInactiveOrders}
	{hideZeroBalanceVaults}
	{activeTokens}
	{selectedChainIds}
	{onWithdrawMultiple}
/>
