<script lang="ts">
	import { transactionStore, InputTokenAmount } from '@rainlanguage/ui-components';
	import type { Hex } from 'viem';
	import {
		getVaultDepositCalldata,
		getVaultWithdrawCalldata,
		type ApprovalCalldataResult,
		type DepositAndAddOrderCalldataResult,
		type Vault
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Button, Toggle } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';

	export let open: boolean;
	export let vault: Vault;
	export let action: 'deposit' | 'withdraw';
	export let subgraphUrl: string;
	export let onDepositOrWithdraw: () => void;

	$: console.log(vault);

	let currentStep = 1;
	let amount: bigint = 0n;
	let isDeposit = true;

	const messages = {
		success: 'Your transaction was successful.',
		pending: 'Processing your transaction...',
		error: 'Transaction failed.'
	};

	// async function executeWalletconnect() {
	// 	isSubmitting = true;
	// 	try {
	// 		if (!$orderbookAddress) throw Error('Select an orderbook to deposit');
	// 		const allowance = await checkAllowance(vault.token.id, $orderbookAddress);
	// 		if (allowance.lt(amount)) {
	// 			const approveCalldata = (await vaultDepositApproveCalldata(
	// 				BigInt(vault.vaultId),
	// 				vault.token.id,
	// 				amount,
	// 				allowance.toBigInt()
	// 			)) as Uint8Array;
	// 			const approveTx = await ethersExecute(approveCalldata, vault.token.id);
	// 			toasts.success('Approve Transaction sent successfully!');
	// 			await approveTx.wait(1);
	// 		}

	// 		const depositCalldata = (await vaultDepositCalldata(
	// 			BigInt(vault.vaultId),
	// 			vault.token.id,
	// 			amount
	// 		)) as Uint8Array;
	// 		const depositTx = await ethersExecute(depositCalldata, $orderbookAddress);
	// 		toasts.success('Transaction sent successfully!');
	// 		await depositTx.wait(1);
	// 		onDeposit();
	// 	} catch (e) {
	// 		reportErrorToSentry(e);
	// 		toasts.error(formatEthersTransactionError(e));
	// 	}
	// 	isSubmitting = false;
	// 	reset();
	// }

	function handleContinue() {
		// const calldata = getVaultDepositCalldata({
		//     subgraphUrl
		// 	deploymentCalldata,
		// 	amount
		// });
		// if (amount > 0n) {
		// 	currentStep = 2;
		// 	transactionStore.handleDepositOrWithdrawTransaction({
		// 		config: $wagmiConfig,
		// 		chainId,
		// 		amount,
		// 		isDeposit
		// 	});
		// }
	}

	function handleClose() {
		open = false;
		currentStep = 1;
		amount = 0n;
	}
</script>

{#if currentStep === 1}
	<Modal bind:open autoclose={false} size="md">
		<div class="space-y-6">
			<div class="flex flex-col gap-4">
				<h3 class="text-xl font-medium">Enter Amount</h3>
				<Toggle bind:checked={isDeposit}>
					{isDeposit ? 'Deposit' : 'Withdraw'}
				</Toggle>
			</div>
			<!-- <TokenAmountInput bind:value={amount} {symbol} {decimals} {maxValue} /> -->
			<div class="flex justify-end gap-2">
				<Button color="alternative" on:click={handleClose}>Cancel</Button>
				<Button color="blue" on:click={handleContinue} disabled={amount <= 0n}>
					{isDeposit ? 'Deposit' : 'Withdraw'}
				</Button>
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} />
{/if}
