<script lang="ts">
	import { Alert, Modal, Button } from 'flowbite-svelte';
	import { ExclamationCircleSolid } from 'flowbite-svelte-icons';
	import { getDeploymentTransactionArgs } from './getDeploymentTransactionArgs';
	import type { Config } from 'wagmi';
	import type { Writable } from 'svelte/store';
	import type {
		ApprovalCalldataResult,
		DepositAndAddOrderCalldataResult,
		DotrainOrderGui,
		OrderIO
	} from '@rainlanguage/orderbook/js_api';
	import type { Hex } from 'viem';
	import type { HandleAddOrderResult } from './getDeploymentTransactionArgs';
	export let open: boolean;
	export let gui: DotrainOrderGui;
	export let allTokenOutputs: OrderIO[];
	export let wagmiConfig: Writable<Config | undefined>;
	export let handleDeployModal: (args: {
		approvals: ApprovalCalldataResult;
		deploymentCalldata: DepositAndAddOrderCalldataResult;
		orderbookAddress: Hex;
		chainId: number;
	}) => void;
	let result: HandleAddOrderResult | null = null;

	let error: string | null = null;
	let errorDetails: string | null = null;
	let deployButtonText: 'Loading...' | 'Deploy' | 'Error' = 'Loading...';

	const handleOpenModal = async () => {
		try {
			result = await getDeploymentTransactionArgs(gui, $wagmiConfig, allTokenOutputs);
			deployButtonText = 'Deploy';
		} catch (e) {
			deployButtonText = 'Error';
			error = 'Error getting deployment transaction data:';
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	};

	$: if (open === true) {
		handleOpenModal();
	}

	async function handleAcceptDisclaimer() {
		if (!result) {
			error = 'No result found';
			return;
		} else {
			open = false;
			handleDeployModal(result);
		}
	}
</script>

<Modal bind:open>
	<div class="flex flex-col items-start gap-y-4">
		<div class="space-y-4">
			<Alert color="red" class="text-base">
				<div class="flex items-center justify-center">
					<ExclamationCircleSolid class="h-6 w-6 text-red-500" />
					<span class="ml-2">
						Before you deploy your strategy, make sure you understand the following...
					</span>
				</div>
			</Alert>
			<ul class="list-outside list-disc space-y-2 text-gray-700">
				<li class="ml-4">
					This front end is provided as a tool to interact with the Raindex smart contracts.
				</li>
				<li class="ml-4">
					You are deploying your own strategy and depositing funds to an immutable smart contract
					using your own wallet and private keys.
				</li>
				<li class="ml-4">
					Nobody is custodying your funds, there is no recourse for recovery of funds if lost.
				</li>
				<li class="ml-4">There is no endorsement or guarantee provided with these strategies.</li>
				<li class="ml-4">
					Do not proceed if you do not understand the strategy you are deploying.
				</li>
				<li class="ml-4">Do not invest unless you are prepared to lose all funds.</li>
			</ul>
		</div>
		<div class="flex gap-2">
			<Button
				size="lg"
				class="w-32"
				color="green"
				disabled={!result}
				on:click={handleAcceptDisclaimer}
			>
				{deployButtonText}
			</Button>
			<Button size="lg" class="w-32" color="red" on:click={() => (open = false)}>Cancel</Button>
		</div>
		<div class="flex flex-col">
			{#if error}
				<span class="ml-2 text-red-500">{error}</span>
			{/if}
			{#if errorDetails}
				<span class="ml-2 text-red-500">{errorDetails}</span>
			{/if}
		</div>
	</div>
</Modal>
