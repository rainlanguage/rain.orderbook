<script lang="ts">
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import DepositInput from './DepositInput.svelte';
	import SelectToken from './SelectToken.svelte';
	import TokenInputOrOutput from './TokenInputOrOutput.svelte';
	import DropdownRadio from '../dropdown/DropdownRadio.svelte';
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
	import { Button, Input, Label, Spinner } from 'flowbite-svelte';
	import { createWalletClient, custom, type Chain } from 'viem';
	import { base, flare, arbitrum, polygon, bsc, mainnet, linea } from 'viem/chains';

	enum DeploymentStepErrors {
		NO_GUI = 'Error loading GUI',
		NO_STRATEGY = 'No valid strategy exists at this URL',
		NO_SELECT_TOKENS = 'Error loading tokens',
		NO_TOKEN_INFO = 'Error loading token information',
		NO_FIELD_DEFINITIONS = 'Error loading field definitions',
		NO_DEPOSITS = 'Error loading deposits',
		NO_TOKEN_INPUTS = 'Error loading token inputs',
		NO_TOKEN_OUTPUTS = 'Error loading token outputs',
		NO_GUI_DETAILS = 'Error getting GUI details'
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

	let dotrain = '';
	let isLoading = false;
	let error: DeploymentStepErrors | null = null;
	let errorDetails: string | null = null;
	let strategyUrl = '';
	let selectTokens: string[] | null = null;
	let allFieldDefinitions: GuiFieldDefinition[] = [];
	let allDepositFields: GuiDeposit[] = [];
	let allTokenOutputs: OrderIO[] = [];
	let allTokensSelected: boolean = false;
	let guiDetails: GuiDetails;

	let inputVaultIds: string[] = [];
	let outputVaultIds: string[] = [];

	async function loadStrategyFromUrl() {
		isLoading = true;
		error = null;
		errorDetails = null;

		try {
			const response = await fetch(strategyUrl);
			if (!response.ok) {
				throw new Error(`HTTP error - status: ${response.status}`);
			}
			dotrain = await response.text();
		} catch (e) {
			error = DeploymentStepErrors.NO_STRATEGY;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
			console.error('Failed to load strategy:', e);
		} finally {
			isLoading = false;
		}
	}

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
			// eslint-disable-next-line no-console
			console.error('Failed to load deployments:', e);
		}
	}

	$: if (dotrain) {
		isLoading = true;
		error = null;
		errorDetails = null;
		gui = null;
		initialize();
		isLoading = false;
	}

	let selectedDeployment: string | undefined = undefined;
	async function handleDeploymentChange(deployment: string) {
		isLoading = true;
		gui = null;
		if (!deployment) return;

		try {
			gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment);
			try {
				selectTokens = gui.getSelectTokens();
				getGuiDetails();
			} catch (e) {
				console.error('ERROR GETTING TOKENS', e);
			}
		} catch (error) {
			// eslint-disable-next-line no-console
			console.error('Failed to get gui:', error);
		}
		isLoading = false;
	}

	$: if (selectedDeployment) {
		handleDeploymentChange(selectedDeployment as string);
	}

	function getGuiDetails() {
		if (!gui) return;
		try {
			guiDetails = gui.getGuiDetails();
		} catch (e) {
			error = DeploymentStepErrors.NO_GUI_DETAILS;
			console.error('Failed to get gui details:', e);
		}
	}

	function getAllFieldDefinitions() {
		if (!gui) return;
		try {
			allFieldDefinitions = gui.getAllFieldDefinitions();
		} catch (e) {
			error = DeploymentStepErrors.NO_FIELD_DEFINITIONS;
			console.error('Failed to get field definitions:', e);
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
			console.error('Failed to get deposits:', e);
		}
	}

	let allTokenInputs: OrderIO[] = [];
	function getAllTokenInputs() {
		if (!gui) return;

		try {
			allTokenInputs = gui.getCurrentDeployment().deployment.order.inputs;
		} catch (e) {
			error = DeploymentStepErrors.NO_TOKEN_INPUTS;
			console.error('Failed to get token inputs:', e);
		}
	}

	function getAllTokenOutputs() {
		if (!gui) return;
		try {
			allTokenOutputs = gui.getCurrentDeployment().deployment.order.outputs;
		} catch (e) {
			error = DeploymentStepErrors.NO_TOKEN_OUTPUTS;
			console.error('Failed to get token outputs:', e);
		}
	}

	$: if (selectTokens?.length === 0 || allTokensSelected) {
		error = null;
		initializeVaultIdArrays();
		getAllDepositFields();
		getAllFieldDefinitions();
		getAllTokenInputs();
		getAllTokenOutputs();
	}

	export function getChainById(chainId: number): Chain {
		const chain = chains[chainId];
		if (!chain) {
			throw new Error(`Unsupported chain ID: ${chainId}`);
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
		} catch (error) {
			// eslint-disable-next-line no-console
			console.error('Failed to add order:', error);
		}
	}

	function initializeVaultIdArrays() {
		if (!gui) return;
		const deployment = gui.getCurrentDeployment();
		inputVaultIds = new Array(deployment.deployment.order.inputs.length).fill('');
		outputVaultIds = new Array(deployment.deployment.order.outputs.length).fill('');
	}
</script>

<div class="mb-4">
	<div class="flex items-end gap-2">
		<div class="flex-1">
			<Input
				id="strategy-url"
				type="url"
				placeholder="Enter URL to .rain file"
				bind:value={strategyUrl}
				size="lg"
			/>
		</div>
		<Button on:click={loadStrategyFromUrl} disabled={!strategyUrl} size="lg">Load Strategy</Button>
	</div>
</div>

{#if error}
	<p class="text-red-500">{error}</p>
{/if}
{#if errorDetails}
	<p class="text-red-500">{errorDetails}</p>
{/if}
{#if dotrain}
	<div class="mb-4">
		<Label class="mb-2 whitespace-nowrap text-xl">Deployments</Label>
		<DropdownRadio options={availableDeployments} bind:value={selectedDeployment}>
			<svelte:fragment slot="content" let:selectedOption let:selectedRef>
				{#if selectedRef === undefined}
					<span>Select a deployment</span>
				{:else if selectedOption?.label}
					<span>{selectedOption.label}</span>
				{:else}
					<span>{selectedRef}</span>
				{/if}
			</svelte:fragment>

			<svelte:fragment slot="option" let:option let:ref>
				<div class="w-full overflow-hidden overflow-ellipsis">
					<div class="text-md break-word">{option.label ? option.label : ref}</div>
				</div>
			</svelte:fragment>
		</DropdownRadio>
	</div>
	{#if isLoading}
		<Spinner />
	{/if}
	{#if gui}
		<div class="flex max-w-2xl flex-col gap-24">
			{#if guiDetails}
				<div class="mt-16 flex max-w-2xl flex-col gap-6 text-start">
					<h1 class="mb-6 text-8xl font-semibold text-gray-900 dark:text-white">
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
