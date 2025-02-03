<script lang="ts">
	import TokenIOSection from './TokenIOSection.svelte';
	import DepositsSection from './DepositsSection.svelte';
	import SelectTokensSection from './SelectTokensSection.svelte';

	import WalletConnect from '../wallet/WalletConnect.svelte';
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type GuiFieldDefinition,
		type NameAndDescription,
		type GuiDeployment,
		type OrderIO,
		type ApprovalCalldataResult,
		type DepositAndAddOrderCalldataResult
	} from '@rainlanguage/orderbook/js_api';
	import { fade } from 'svelte/transition';
	import { Accordion, Button } from 'flowbite-svelte';
	import { getAccount, type Config } from '@wagmi/core';
	import { type Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import type { Hex } from 'viem';
	import FieldDefinitionsSection from './FieldDefinitionsSection.svelte';

	enum DeploymentStepErrors {
		NO_GUI = 'Error loading GUI',
		NO_STRATEGY = 'No valid strategy exists at this URL',
		NO_SELECT_TOKENS = 'Error loading tokens',
		NO_TOKEN_INFO = 'Error loading token information',
		NO_FIELD_DEFINITIONS = 'Error loading field definitions',
		NO_DEPOSITS = 'Error loading deposits',
		NO_TOKEN_INPUTS = 'Error loading token inputs',
		NO_TOKEN_OUTPUTS = 'Error loading token outputs',
		NO_GUI_DETAILS = 'Error getting GUI details',
		NO_CHAIN = 'Unsupported chain ID',
		ADD_ORDER_FAILED = 'Failed to add order'
	}

	export let dotrain: string;
	export let deployment: string;
	export let deploymentDetails: NameAndDescription;
	export let handleDeployModal: (args: {
		approvals: ApprovalCalldataResult;
		deploymentCalldata: DepositAndAddOrderCalldataResult;
		orderbookAddress: Hex;
		chainId: number;
	}) => void;

	let error: DeploymentStepErrors | null = null;
	let errorDetails: string | null = null;
	let selectTokens: string[] | null = null;
	let allDepositFields: GuiDeposit[] = [];
	let allTokenOutputs: OrderIO[] = [];
	let allFieldDefinitions: GuiFieldDefinition[] = [];
	let allTokensSelected: boolean = false;
	let inputVaultIds: string[] = [];
	let outputVaultIds: string[] = [];

	let gui: DotrainOrderGui | null = null;
	let addOrderError: DeploymentStepErrors | null = null;
	let addOrderErrorDetails: string | null = null;
	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;

	$: if (deployment) {
		handleDeploymentChange(deployment);
	}

	async function handleDeploymentChange(deployment: string) {
		if (!deployment || !dotrain) return;
		error = null;
		errorDetails = null;

		try {
			gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment);

			if (gui) {
				try {
					selectTokens = await gui.getSelectTokens();
					return selectTokens;
				} catch (e) {
					error = DeploymentStepErrors.NO_SELECT_TOKENS;
					return (errorDetails = e instanceof Error ? e.message : 'Unknown error');
				}
			}
		} catch (e) {
			error = DeploymentStepErrors.NO_GUI;
			return (errorDetails = e instanceof Error ? e.message : 'Unknown error');
		}
	}

	function getAllFieldDefinitions() {
		if (!gui) return;
		try {
			allFieldDefinitions = gui.getAllFieldDefinitions();
		} catch (e) {
			error = DeploymentStepErrors.NO_FIELD_DEFINITIONS;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	async function getAllDepositFields() {
		if (!gui) return;
		try {
			let dep: GuiDeployment = gui.getCurrentDeployment();
			let depositFields: GuiDeposit[] = dep.deposits;

			allDepositFields = depositFields;
		} catch (e) {
			error = DeploymentStepErrors.NO_DEPOSITS;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	let allTokenInputs: OrderIO[] = [];
	function getAllTokenInputs() {
		if (!gui) return;

		try {
			allTokenInputs = gui.getCurrentDeployment().deployment.order.inputs;
		} catch (e) {
			error = DeploymentStepErrors.NO_TOKEN_INPUTS;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	function getAllTokenOutputs() {
		if (!gui) return;
		try {
			allTokenOutputs = gui.getCurrentDeployment().deployment.order.outputs;
		} catch (e) {
			error = DeploymentStepErrors.NO_TOKEN_OUTPUTS;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	$: if (selectTokens?.length === 0 || allTokensSelected) {
		updateFields();
	}

	async function updateFields() {
		try {
			error = null;
			errorDetails = null;
			initializeVaultIdArrays();
			getAllDepositFields();
			getAllFieldDefinitions();
			getAllTokenInputs();
			getAllTokenOutputs();
		} catch (e) {
			error = DeploymentStepErrors.NO_GUI;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	async function handleAddOrder() {
		try {
			if (!gui || !$wagmiConfig) return;
			const { address } = getAccount($wagmiConfig);
			if (!address) return;
			let approvals = await gui.generateApprovalCalldatas(address);
			const deploymentCalldata = await gui.generateDepositAndAddOrderCalldatas();
			const chainId = gui.getCurrentDeployment().deployment.order.network['chain-id'] as number;
			// @ts-expect-error orderbook is not typed
			const orderbookAddress = gui.getCurrentDeployment().deployment.order.orderbook.address;
			const outputTokenInfos = await Promise.all(
				allTokenOutputs.map((token) => gui?.getTokenInfo(token.token?.key as string))
			);

			approvals = approvals.map((approval) => {
				const token = outputTokenInfos.find((token) => token?.address === approval.token);
				return {
					...approval,
					symbol: token?.symbol
				};
			});

			handleDeployModal({
				approvals,
				deploymentCalldata,
				orderbookAddress,
				chainId
			});
		} catch (e) {
			addOrderError = DeploymentStepErrors.ADD_ORDER_FAILED;
			addOrderErrorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	function initializeVaultIdArrays() {
		if (!gui) return;
		const deployment = gui.getCurrentDeployment();
		inputVaultIds = new Array(deployment.deployment.order.inputs.length).fill('');
		outputVaultIds = new Array(deployment.deployment.order.outputs.length).fill('');
	}
</script>

<div>
	{#if error}
		<p class="text-red-500">{error}</p>
	{/if}
	{#if errorDetails}
		<p class="text-red-500">{errorDetails}</p>
	{/if}
	{#if dotrain}
		{#if gui}
			<div class="flex max-w-3xl flex-col gap-24" in:fade>
				{#if deploymentDetails}
					<div class="mt-16 flex max-w-2xl flex-col gap-4 text-start">
						<h1 class=" text-4xl font-semibold text-gray-900 lg:text-8xl dark:text-white">
							{deploymentDetails.name}
						</h1>
						<p class="text-2xl text-gray-600 lg:text-3xl dark:text-gray-400">
							{deploymentDetails.description}
						</p>
					</div>
				{/if}

				{#if selectTokens && selectTokens.length > 0}
					<SelectTokensSection bind:gui bind:selectTokens bind:allTokensSelected />
				{/if}

				{#if allTokensSelected || selectTokens?.length === 0}
					<Accordion multiple={true}>
						{#if allFieldDefinitions.length > 0}
							<FieldDefinitionsSection bind:allFieldDefinitions bind:gui />
						{/if}

						{#if allDepositFields.length > 0}
							<DepositsSection bind:allDepositFields bind:gui />
						{/if}

						{#if allTokenInputs.length > 0 && allTokenOutputs.length > 0}
							<TokenIOSection
								bind:allTokenInputs
								bind:allTokenOutputs
								bind:gui
								bind:inputVaultIds
								bind:outputVaultIds
							/>
						{/if}
					</Accordion>

					<div class="flex flex-col gap-2">
						{#if $wagmiConnected}
							<Button size="lg" on:click={handleAddOrder}>Deploy Strategy</Button>
						{:else}
							<WalletConnect {appKitModal} connected={wagmiConnected} />
						{/if}
						<div class="flex flex-col">
							{#if addOrderError}
								<p class="text-red-500">{addOrderError}</p>
							{/if}
							{#if addOrderErrorDetails}
								<p class="text-red-500">{addOrderErrorDetails}</p>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</div>
