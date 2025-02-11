<script lang="ts">
	import TokenIOSection from './TokenIOSection.svelte';
	import DepositsSection from './DepositsSection.svelte';
	import SelectTokensSection from './SelectTokensSection.svelte';
	import ComposedRainlangModal from './ComposedRainlangModal.svelte';
	import FieldDefinitionsSection from './FieldDefinitionsSection.svelte';
	import WalletConnect from '../wallet/WalletConnect.svelte';
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type GuiFieldDefinition,
		type NameAndDescription,
		type GuiDeployment,
		type OrderIO,
		type ApprovalCalldataResult,
		type DepositAndAddOrderCalldataResult,
		DotrainOrder
	} from '@rainlanguage/orderbook/js_api';
	import { fade } from 'svelte/transition';
	import { Button, Toggle } from 'flowbite-svelte';
	import { getAccount, type Config } from '@wagmi/core';
	import { type Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import type { Hex } from 'viem';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

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
		SERIALIZE_ERROR = 'Error serializing state',
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
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;
	let selectTokens: string[] | null = null;
	let allDepositFields: GuiDeposit[] = [];
	let allTokenOutputs: OrderIO[] = [];
	let allFieldDefinitions: GuiFieldDefinition[] = [];
	let allTokensSelected: boolean = false;
	let showAdvancedOptions: boolean = false;
	let gui: DotrainOrderGui | null = null;
	let error: DeploymentStepErrors | null = null;
	let errorDetails: string | null = null;

	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let stateFromUrl: string | null = null;

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
			error = DeploymentStepErrors.ADD_ORDER_FAILED;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	async function handleShareChoices() {
		// copy the current url to the clipboard
		navigator.clipboard.writeText($page.url.toString());
	}

	onMount(async () => {
		if ($page.url.searchParams) {
			if (stateFromUrl) {
				await handleGetStateFromUrl();
			}
		}
	});

	async function handleGetStateFromUrl() {
		if (!$page.url.searchParams.get('state')) return;
		gui = await DotrainOrderGui.deserializeState(
			dotrain,
			$page.url.searchParams.get('state') || ''
		);
		if (gui) {
			try {
				selectTokens = await gui.getSelectTokens();
				if (selectTokens?.every((t) => gui?.isSelectTokenSet(t))) {
					allTokensSelected = true;
				}
				// if we have deposits or vault ids set, show advanced options
				const deposits = gui?.getDeposits();
				const inputVaultIds = gui
					?.getCurrentDeployment()
					?.deployment?.order?.inputs.map((input) => input.vaultId);
				const outputVaultIds = gui
					?.getCurrentDeployment()
					?.deployment?.order?.outputs.map((output) => output.vaultId);
				if (deposits || inputVaultIds || outputVaultIds) {
					showAdvancedOptions = true;
				}
			} catch (e) {
				error = DeploymentStepErrors.NO_SELECT_TOKENS;
				return (errorDetails = e instanceof Error ? e.message : 'Unknown error');
			}
		}
	}

	async function composeRainlang() {
		if (!gui) return;
		gui.updateScenarioBindings();
		const deployment = gui.getCurrentDeployment();
		const dotrain = gui.generateDotrainText();
		const dotrainOrder = await DotrainOrder.create(dotrain);
		const composedRainlang = await dotrainOrder.composeDeploymentToRainlang(deployment.key);
		return composedRainlang;
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
			<div class="flex max-w-3xl flex-col gap-12" in:fade>
				{#if deploymentDetails}
					<div class="mt-8 flex max-w-2xl flex-col gap-4 text-start">
						<h1 class=" text-3xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
							{deploymentDetails.name}
						</h1>
						<p class="text-xl text-gray-600 lg:text-2xl dark:text-gray-400">
							{deploymentDetails.description}
						</p>
					</div>
				{/if}

				{#if selectTokens && selectTokens.length > 0}
					<SelectTokensSection {gui} {selectTokens} bind:allTokensSelected {handleUpdateGuiState} />
				{/if}

				{#if allTokensSelected || selectTokens?.length === 0}
					{#if allFieldDefinitions.length > 0}
						<FieldDefinitionsSection {allFieldDefinitions} {gui} {handleUpdateGuiState} />
					{/if}

					<Toggle bind:checked={showAdvancedOptions}>Show advanced options</Toggle>

					{#if allDepositFields.length > 0 && showAdvancedOptions}
						<DepositsSection bind:allDepositFields {gui} {handleUpdateGuiState} />
					{/if}

					{#if allTokenInputs.length > 0 && allTokenOutputs.length > 0 && showAdvancedOptions}
						<TokenIOSection bind:allTokenInputs bind:allTokenOutputs {gui} {handleUpdateGuiState} />
					{/if}

					<div class="flex flex-col gap-2">
						<Button
							size="lg"
							class="flex gap-2"
							color="alternative"
							data-testid="review-choices-button"
							on:click={handleShareChoices}>Share these choices</Button
						>
						{#if $wagmiConnected}
							<ComposedRainlangModal {composeRainlang} />
							<Button size="lg" on:click={handleAddOrder}>Deploy Strategy</Button>
						{:else}
							<WalletConnect {appKitModal} connected={wagmiConnected} />
						{/if}

						<div class="flex flex-col">
							{#if error}
								<p class="text-red-500">{error}</p>
							{/if}
							{#if errorDetails}
								<p class="text-red-500">{errorDetails}</p>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</div>
