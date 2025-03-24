<script lang="ts">
	import {
		transactionStore,
		InputTokenAmount,
		WalletConnect,
		type DepositOrWithdrawArgs
	} from '@rainlanguage/ui-components';
	import {
		type WithdrawCalldataResult,
		getVaultWithdrawCalldata
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Button, Badge } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';
	import { appKitModal, connected, signerAddress } from '$lib/stores/wagmi';
	import { formatUnits, type Hex } from 'viem';

	export let open: boolean;
	export let args: DepositOrWithdrawArgs;

	const { vault, chainId, subgraphUrl } = args;

	let currentStep = 1;
	let amount: bigint = 0n;
	let errorMessage = '';
	let withdrawCalldata: WithdrawCalldataResult | undefined = undefined;
	let isCheckingCalldata = false;

	const messages = {
		success: 'Transaction successful.',
		pending: 'Processing your transaction...',
		error: 'Transaction failed.'
	};

	async function handleTransaction(transactionCalldata: WithdrawCalldataResult) {
		transactionStore.handleDepositOrWithdrawTransaction({
			config: $wagmiConfig,
			transactionCalldata,
			action: 'withdraw',
			chainId,
			vault,
			subgraphUrl
		});
	}

	async function handleContinue() {
		isCheckingCalldata = true;
		try {
			withdrawCalldata = await getVaultWithdrawCalldata(vault, amount.toString());
			if (withdrawCalldata) {
				handleTransaction(withdrawCalldata);
			}
			currentStep = 2;
		} catch {
			errorMessage = 'Failed to get calldata.';
		} finally {
			isCheckingCalldata = false;
		}
	}

	function handleClose() {
		transactionStore.reset();
		open = false;
		currentStep = 1;
		amount = 0n;
	}

	$: amountGreaterThanBalance = amount > BigInt(vault.balance);
</script>

{#if currentStep === 1}
	<Modal bind:open autoclose={false} size="md">
		<div class="space-y-6">
			<div class="flex flex-col gap-4">
				<h3 class="text-xl font-medium">Enter Withdraw Amount</h3>
			</div>
			<div class="flex flex-col gap-2">
				<Badge color="yellow" class="w-fit" data-testid="balance-badge">
					Vault balance: {formatUnits(BigInt(vault.balance), Number(vault.token.decimals))}
					{vault.token.symbol}
				</Badge>

				<InputTokenAmount
					bind:value={amount}
					symbol={vault.token.symbol}
					decimals={Number(vault.token.decimals)}
					maxValue={BigInt(vault.balance)}
				/>
			</div>
			<div class="flex flex-col justify-end gap-2">
				<div class="flex gap-2">
					<Button color="alternative" on:click={handleClose}>Cancel</Button>
					{#if $signerAddress}
						<div class="flex flex-col gap-2">
							<Button
								color="blue"
								on:click={handleContinue}
								disabled={amount <= 0n || amountGreaterThanBalance || isCheckingCalldata}
							>
								{#if isCheckingCalldata}
									Checking...
								{:else}
									Withdraw
								{/if}
							</Button>
						</div>
					{:else}
						<WalletConnect {appKitModal} {connected} {signerAddress} />
					{/if}
				</div>
				{#if errorMessage}
					<p data-testid="error-message">{errorMessage}</p>
				{/if}
				{#if amountGreaterThanBalance}
					<p class="text-red-500" data-testid="error">Amount cannot exceed available balance.</p>
				{/if}
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} />
{/if}
