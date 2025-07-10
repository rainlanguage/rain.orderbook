<script lang="ts">
	import { PageHeader, useAccount, useToasts, useTransactions } from '@rainlanguage/ui-components';
	import { page } from '$app/stores';
	import { VaultDetail } from '@rainlanguage/ui-components';
	import {
		handleDepositModal,
		handleTransactionConfirmationModal,
		handleWithdrawModal
	} from '$lib/services/modal';
	import { RaindexClient, type Address, type RaindexVault } from '@rainlanguage/orderbook';
	import type { Hex } from 'viem';
	import { lightweightChartsTheme } from '$lib/darkMode';
	import { handleVaultWithdraw } from '$lib/services/handleVaultWithdraw';
	import { handleVaultDeposit } from '$lib/services/handleVaultDeposit';

	const { id, chainId, orderbook } = $page.params;
	const parsedId = id as Hex;
	const parsedChainId = Number(chainId);
	const orderbookAddress = orderbook as Address;

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

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

<PageHeader title="Vault" pathname={$page.url.pathname} />

<VaultDetail
	id={parsedId}
	{orderbookAddress}
	chainId={parsedChainId}
	{lightweightChartsTheme}
	{onDeposit}
	{onWithdraw}
/>
