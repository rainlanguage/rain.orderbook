<script lang="ts">
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import DepositInput from './DepositInput.svelte';
	import SelectToken from './SelectToken.svelte';
	import TokenInputOrOutput from './TokenInputOrOutput.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import {
		DotrainOrderGui,
		type DeploymentKeys,
		type ApprovalCalldataResult,
		type DepositAndAddOrderCalldataResult,
		type GuiDeposit,
		type GuiFieldDefinition,
		type GuiDetails,
		type GuiDeployment,
		type OrderIO
	} from '@rainlanguage/orderbook/js_api';
	import { Button, Spinner } from 'flowbite-svelte';
	import { createWalletClient, custom, type Chain } from 'viem';
	import { base, flare, arbitrum, polygon, bsc, mainnet, linea } from 'viem/chains';
	import { fade } from 'svelte/transition';
	import { writable } from 'svelte/store';

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

	const chains: Record<number, Chain> = {
		[base.id]: base,
		[flare.id]: flare,
		[arbitrum.id]: arbitrum,
		[polygon.id]: polygon,
		[bsc.id]: bsc,
		[mainnet.id]: mainnet,
		[linea.id]: linea
	};

	export let dotrain;

	let error: DeploymentStepErrors | null = null;
	let errorDetails: string | null = null;
	let selectTokens: string[] | null = null;
	let allFieldDefinitions: GuiFieldDefinition[] = [];
	let allDepositFields: GuiDeposit[] = [];
	let allTokenOutputs: OrderIO[] = [];
	let allTokensSelected: boolean = false;
	let guiDetails: GuiDetails;
	let inputVaultIds: string[] = [];
	let outputVaultIds: string[] = [];
	let gui: DotrainOrderGui | null = null;
	let availableDeployments: Record<string, { label: string }> = {};

	async function initialize() {
		try {
			let deployments: DeploymentKeys = await DotrainOrderGui.getDeploymentKeys(dotrain);
			availableDeployments = Object.fromEntries(
				deployments.map((deployment) => [
					deployment,
					{
						label: deployment
					}
				])
			);
		} catch (e: unknown) {
			error = DeploymentStepErrors.NO_GUI;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}
	}

	$: if (dotrain) {
		initializeAsync();
	}

	async function initializeAsync() {
		error = null;
		errorDetails = null;
		gui = null;
		await initialize();
	}

	async function handleDeploymentChange(deployment: string) {
		if (!deployment || !dotrain) return;
		error = null;
		errorDetails = null;

		try {
			gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment);

			if (gui) {
				try {
					await getGuiDetails();
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

	async function getGuiDetails() {
		if (!gui) return;
		try {
			guiDetails = gui.getGuiDetails();
		} catch (e) {
			error = DeploymentStepErrors.NO_GUI_DETAILS;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
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

	export function getChainById(chainId: number): Chain {
		const chain = chains[chainId];
		if (!chain) {
			error = DeploymentStepErrors.NO_CHAIN;
			errorDetails = `Unsupported chain ID: ${chainId}`;
		}
		return chain;
	}

	async function handleAddOrder() {
		try {
			if (!gui) return;

			// @ts-expect-error window.ethereum is not typed
			await window.ethereum?.request({ method: 'eth_requestAccounts' });
			const walletClient = createWalletClient({
				chain: getChainById(
					gui.getCurrentDeployment().deployment.order.network['chain-id'] as number
				),
				// @ts-expect-error window.ethereum is not typed
				transport: custom(window.ethereum!)
			});
			const [account] = await walletClient.getAddresses();

			const approvals: ApprovalCalldataResult = await gui.generateApprovalCalldatas(account);
			for (const approval of approvals) {
				await walletClient.sendTransaction({
					account,
					to: approval.token as `0x${string}`,
					data: approval.calldata as `0x${string}`
				});
			}

			const calldata: DepositAndAddOrderCalldataResult =
				await gui.generateDepositAndAddOrderCalldatas();
			await walletClient.sendTransaction({
				account,
				// @ts-expect-error orderbook is not typed
				to: gui.getCurrentDeployment().deployment.order.orderbook.address as `0x${string}`,
				data: calldata as `0x${string}`
			});
		} catch (e) {
			error = DeploymentStepErrors.ADD_ORDER_FAILED;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
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
		{$initializingFields}
		<DeploymentSectionHeader title="Select Deployment" />

		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6">
			{#each Object.entries(availableDeployments) as [deployment, { label }]}
				<Button on:click={handleDeploymentChange(deployment)}>{label}</Button>
				{#if $initializingFields === true}
					<div class="flex flex-col items-center justify-center gap-4 p-8">
						<h1>Loading...</h1>
						<Spinner />
					</div>
				{/if}
			{/each}
		</div>
		{#if gui}
			<div class="flex max-w-2xl flex-col gap-24" in:fade>
				{#if guiDetails}
					<div class="mt-16 flex max-w-2xl flex-col gap-6 text-start">
						<h1 class="mb-6 text-4xl font-semibold text-gray-900 lg:text-8xl dark:text-white">
							{guiDetails.name}
						</h1>
						<p class="text-xl text-gray-600 dark:text-gray-400">
							{guiDetails.description}
						</p>
					</div>
				{/if}

				{#if selectTokens && selectTokens.length > 0}
					<div class="flex w-full flex-col gap-6">
						<DeploymentSectionHeader
							title="Select Tokens"
							description="Select the tokens that you want to use in your order."
						/>
						<div class="flex w-full flex-col gap-4">
							{#each selectTokens as tokenKey}
								<SelectToken {tokenKey} bind:gui bind:selectTokens bind:allTokensSelected />
							{/each}
						</div>
					</div>
				{/if}

				{#if allTokensSelected || selectTokens?.length === 0}
					{#if allFieldDefinitions.length > 0}
						<div class="flex w-full flex-col items-center gap-24">
							{#each allFieldDefinitions as fieldDefinition}
								<FieldDefinitionInput {fieldDefinition} {gui} />
							{/each}
						</div>
					{/if}

					{#if allDepositFields.length > 0}
						<div class="flex w-full flex-col items-center gap-24">
							{#each allDepositFields as deposit}
								<DepositInput bind:deposit bind:gui />
							{/each}
						</div>
					{/if}
					{#if allTokenInputs.length > 0 && allTokenOutputs.length > 0}
						<div class="flex w-full flex-col gap-6">
							<DeploymentSectionHeader
								title={'Input/Output Vaults'}
								description={'The vault addresses for the input and output tokens.'}
							/>
							{#if allTokenInputs.length > 0}
								{#each allTokenInputs as input, i}
									<TokenInputOrOutput
										{i}
										label="Input"
										vault={input}
										vaultIds={inputVaultIds}
										{gui}
									/>
								{/each}
							{/if}

							{#if allTokenOutputs.length > 0}
								{#each allTokenOutputs as output, i}
									<TokenInputOrOutput
										{i}
										label="Output"
										vault={output}
										vaultIds={outputVaultIds}
										{gui}
									/>
								{/each}
							{/if}
						</div>
					{/if}
					<Button size="lg" on:click={handleAddOrder}>Deploy Strategy</Button>
				{/if}
			</div>
		{/if}
	{/if}
</div>
