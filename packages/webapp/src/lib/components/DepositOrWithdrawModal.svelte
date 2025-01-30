<script lang="ts">
	import { transactionStore, InputTokenAmount } from '@rainlanguage/ui-components';
	import {
		getVaultDepositCalldata,
		getVaultApprovalCalldata,
		type DepositCalldataResult,
		type Vault,
		type ApprovalCalldata
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Button, Toggle } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';

	export let open: boolean;
	export let vault: Vault;
	export let chainId: number;
	export let rpcUrl: string;
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

	async function handleContinue() {
		let approvalCalldata: ApprovalCalldata | undefined = undefined;
		try {
			approvalCalldata = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
			console.log('approval calldata!', approvalCalldata);
		} catch (e) {
			console.error('error getting approval calldata!', e);
			approvalCalldata = undefined;
		}
		const depositCalldata: DepositCalldataResult = await getVaultDepositCalldata(
			vault.token.address,
			vault.vaultId,
			amount.toString()
		);
		currentStep = 2;
		transactionStore.handleDepositOrWithdrawTransaction({
			config: $wagmiConfig,
			depositCalldata,
			approvalCalldata,
			chainId,
			vault
		});
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
			<InputTokenAmount
				bind:value={amount}
				symbol={vault.token.symbol}
				decimals={Number(vault.token.decimals)}
				maxValue={0n}
			/>
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
