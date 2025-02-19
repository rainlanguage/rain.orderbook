<script lang="ts">
	import { Alert } from 'flowbite-svelte';
	import TokenIOSection from './TokenIOSection.svelte';
	import DepositsSection from './DepositsSection.svelte';
	import SelectTokensSection from './SelectTokensSection.svelte';
	import ComposedRainlangModal from './ComposedRainlangModal.svelte';
	import FieldDefinitionsSection from './FieldDefinitionsSection.svelte';
	import { type ConfigSource } from '../../typeshare/config';
	import WalletConnect from '../wallet/WalletConnect.svelte';
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type GuiFieldDefinition,
		type GuiDeployment,
		type OrderIO,
		type SelectTokens,
		type NameAndDescription,
		type AllTokenInfos
	} from '@rainlanguage/orderbook/js_api';
	import { fade } from 'svelte/transition';
	import { Button, Toggle, Spinner } from 'flowbite-svelte';
	import { type Config } from '@wagmi/core';
	import { type Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import ShareChoicesButton from './ShareChoicesButton.svelte';
	import { handleShareChoices } from '$lib/services/handleShareChoices';
	import DisclaimerModal from './DisclaimerModal.svelte';
	import type { ComponentProps } from 'svelte';
	import type { DeploymentArgs } from '$lib/types/transaction';
	import { getDeploymentTransactionArgs } from './getDeploymentTransactionArgs';
	import type { HandleAddOrderResult } from './getDeploymentTransactionArgs';
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
	export let settings: Writable<ConfigSource>;
	export let dotrain: string;
	export let deployment: GuiDeployment;
	export let strategyDetail: NameAndDescription;

	export let handleDeployModal: (args: DeploymentArgs) => void;
	export let handleDisclaimerModal: (args: Omit<ComponentProps<DisclaimerModal>, 'open'>) => void;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;

	let selectTokens: SelectTokens | null = null;
	let allDepositFields: GuiDeposit[] = [];
	let allTokenOutputs: OrderIO[] = [];
	let allFieldDefinitions: GuiFieldDefinition[] = [];
	let allTokensSelected: boolean = false;
	let showAdvancedOptions: boolean = false;
	let gui: DotrainOrderGui | null = null;
	let checkingDeployment: boolean = false;
	let error: DeploymentStepErrors | null = null;
	let errorDetails: string | null = null;
	let networkKey: string | null = null;
	let subgraphUrl: string = '';
	let allTokenInfos: AllTokenInfos = [];

	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let stateFromUrl: string | null = null;
	$: if (deployment) {
		handleDeploymentChange(deployment.key);
	}

	async function handleDeploymentChange(deployment: string) {
		if (!deployment || !dotrain) return;
		error = null;
		errorDetails = null;

		try {
			gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment);

			if (gui) {
				networkKey = await gui.getNetworkKey();
				subgraphUrl = $settings?.subgraphs?.[networkKey] ?? '';
				try {
					selectTokens = gui.getSelectTokens();
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

	async function _handleShareChoices() {
		if (!gui) return;
		await handleShareChoices(gui);
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
		areAllTokensSelected();
	}

	async function _handleUpdateGuiState(gui: DotrainOrderGui) {
		await areAllTokensSelected();

		if (allTokensSelected) {
			let newAllTokenInfos = await gui.getAllTokenInfos();
			if (newAllTokenInfos !== allTokenInfos) {
				allTokenInfos = newAllTokenInfos;
				updateFields();
			}
		}

		handleUpdateGuiState(gui);
	}

	async function handleDeployButtonClick() {
		error = null;
		errorDetails = null;

		if (!gui) {
			error = DeploymentStepErrors.NO_GUI;
			return;
		}
		if (!allTokenOutputs) {
			error = DeploymentStepErrors.NO_TOKEN_OUTPUTS;
			return;
		}
		if (!wagmiConfig) {
			error = DeploymentStepErrors.NO_CHAIN;
			return;
		}

		if (!networkKey) {
			error = DeploymentStepErrors.NO_CHAIN;
			return;
		}

		let result: HandleAddOrderResult | null = null;

		checkingDeployment = true;

		try {
			result = await getDeploymentTransactionArgs(gui, $wagmiConfig);
		} catch (e) {
			checkingDeployment = false;
			error = DeploymentStepErrors.ADD_ORDER_FAILED;
			errorDetails = e instanceof Error ? e.message : 'Unknown error';
		}

		if (!result) {
			checkingDeployment = false;
			error = DeploymentStepErrors.ADD_ORDER_FAILED;
			return;
		}

		checkingDeployment = false;

		const onAccept = () => {
			if (!networkKey) {
				error = DeploymentStepErrors.NO_CHAIN;
				return;
			}

			handleDeployModal({
				...result,
				subgraphUrl: subgraphUrl,
				network: networkKey
			});
		};

		handleDisclaimerModal({ onAccept });
	}

	const areAllTokensSelected = async () => {
		if (gui) {
			try {
				allTokensSelected = gui.areAllTokensSelected();
				if (!allTokensSelected) return;

				allTokenInfos = await gui.getAllTokenInfos();

				// if we have deposits or vault ids set, show advanced options
				const hasDeposits = gui.hasAnyDeposit();
				const hasVaultIds = gui.hasAnyVaultId();
				if (hasDeposits || hasVaultIds) {
					showAdvancedOptions = true;
				}
			} catch (e) {
				error = DeploymentStepErrors.NO_SELECT_TOKENS;
				return (errorDetails = e instanceof Error ? e.message : 'Unknown error');
			}
		}
	};
</script>

<div>
	{#if error || errorDetails}
		<Alert color="red">
			{#if error}
				<p class="text-red-500">{error}</p>
			{/if}
			{#if errorDetails}
				<p class="text-red-500">{errorDetails}</p>
			{/if}
		</Alert>
	{/if}
	{#if dotrain}
		{#if gui}
			<div class="flex max-w-3xl flex-col gap-12" in:fade>
				{#if deployment}
					<div class="flex max-w-2xl flex-col gap-4 text-start">
						<h1 class=" text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
							{strategyDetail.name}
						</h1>
						<p class="text-xl text-gray-600 lg:text-2xl dark:text-gray-400">
							{deployment.description}
						</p>
					</div>
				{/if}

				{#if selectTokens && selectTokens.length > 0}
					<SelectTokensSection {gui} {selectTokens} handleUpdateGuiState={_handleUpdateGuiState} />
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

					{#if error || errorDetails}
						<Alert color="red">
							{#if error}
								<p class="text-red-500">{error}</p>
							{/if}
							{#if errorDetails}
								<p class="text-red-500">{errorDetails}</p>
							{/if}
						</Alert>
					{/if}

					<div class="flex flex-wrap items-start justify-start gap-2">
						{#if $wagmiConnected}
							<Button
								size="lg"
								on:click={handleDeployButtonClick}
								class="bg-gradient-to-br from-blue-600 to-violet-600"
							>
								{#if checkingDeployment}
									<Spinner size="4" color="white" />
									<span class="ml-2">Checking deployment...</span>
								{:else}
									Deploy Strategy
								{/if}
							</Button>
						{:else}
							<WalletConnect {appKitModal} connected={wagmiConnected} />
						{/if}
						<ComposedRainlangModal {gui} />
						<ShareChoicesButton handleShareChoices={_handleShareChoices} />
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</div>
