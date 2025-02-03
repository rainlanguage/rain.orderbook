<script lang="ts">
	import { transactionStore, InputTokenAmount, WalletConnect } from '@rainlanguage/ui-components';
	import {
		getVaultDepositCalldata,
		getVaultApprovalCalldata,
		type DepositCalldataResult,
		type WithdrawCalldataResult,
		type Vault,
		type ApprovalCalldata,
		getVaultWithdrawCalldata
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import { Modal, Button } from 'flowbite-svelte';
	import TransactionModal from './TransactionModal.svelte';
	import { appKitModal, connected, signerAddress } from '$lib/stores/wagmi';
	import { readContract } from '@wagmi/core';
	import { erc20Abi, type Hex } from 'viem';

	export let open: boolean;
	export let action: 'deposit' | 'withdraw';
	export let vault: Vault;
	export let chainId: number;
	export let rpcUrl: string;
	export let onDepositOrWithdraw: () => void;

	function handleSuccess() {
		setTimeout(() => {
			onDepositOrWithdraw();
		}, 5000);
	}

	let currentStep = 1;
	let amount: bigint = 0n;
	let userBalance: bigint = 0n;

	const messages = {
		success: 'Your transaction was successful.',
		pending: 'Processing your transaction...',
		error: 'Transaction failed.'
	};

	$: if ($signerAddress) {
		getUserBalance();
	}

	const getUserBalance = async () => {
		userBalance = await readContract($wagmiConfig, {
			abi: erc20Abi,
			address: vault.token.address as Hex,
			functionName: 'balanceOf',
			args: [$signerAddress as Hex]
		});
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
				transactionCalldata: depositCalldata,
				action,
				approvalCalldata,
				chainId,
				vault
			});
		} else if (action === 'withdraw') {
			const withdrawCalldata: WithdrawCalldataResult = await getVaultWithdrawCalldata(
				vault,
				amount.toString()
			);
			currentStep = 2;
			transactionStore.handleDepositOrWithdrawTransaction({
				config: $wagmiConfig,
				transactionCalldata: withdrawCalldata,
				action,
				chainId,
				vault
			});
		}
	}

	function handleClose() {
		transactionStore.reset();
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
				maxValue={userBalance}
			/>
			<div class="flex justify-end gap-2">
				<Button color="alternative" on:click={handleClose}>Cancel</Button>
				{#if $signerAddress}
					<!-- <Button on:click={setValueToMax} color="blue">Max</Button> -->
					<Button color="blue" on:click={handleContinue} disabled={amount <= 0n}>
						{action === 'deposit' ? 'Deposit' : 'Withdraw'}
					</Button>
				{:else}
					<WalletConnect {appKitModal} {connected} />
				{/if}
			</div>
		</div>
	</Modal>
{:else}
	<TransactionModal bind:open {messages} on:close={handleClose} on:success={handleSuccess} />
{/if}
