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
	import { Modal, Button } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';

	export let open: boolean;
	export let action: 'deposit' | 'withdraw';
	export let vault: Vault;
	export let chainId: number;
	export let rpcUrl: string;
	export let onDepositOrWithdraw: () => void;

	let currentStep = 1;
	let amount: bigint = 0n;

	const messages = {
		success: 'Your transaction was successful.',
		pending: 'Processing your transaction...',
		error: 'Transaction failed.'
	};

	async function handleContinue() {
		if (action === 'deposit') {
			let approvalCalldata: ApprovalCalldata | undefined = undefined;
			try {
				approvalCalldata = await getVaultApprovalCalldata(rpcUrl, vault, amount.toString());
			} catch {
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
		} else {
			console.log('withdraw');
		}
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
					{action === 'deposit' ? 'Deposit' : 'Withdraw'}
				</Button>
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} on:success={onDepositOrWithdraw} />
{/if}
