<script lang="ts">
	import { Alert } from 'flowbite-svelte';
	import TokenIOInput from './TokenIOInput.svelte';
	import ComposedRainlangModal from './ComposedRainlangModal.svelte';
	import { type ConfigSource, type TokenInfo } from '@rainlanguage/orderbook/js_api';
	import WalletConnect from '../wallet/WalletConnect.svelte';
	import {
		type GuiDepositCfg,
		type GuiFieldDefinitionCfg,
		type NameAndDescriptionCfg,
		type OrderIOCfg
	} from '@rainlanguage/orderbook/js_api';
	import { fade } from 'svelte/transition';
	import { Button, Toggle, Spinner } from 'flowbite-svelte';
	import { type Config } from '@wagmi/core';
	import { type Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import ShareChoicesButton from './ShareChoicesButton.svelte';
	import { handleShareChoices } from '../../services/handleShareChoices';
	import type { DisclaimerModalProps, DeployModalProps } from '../../types/modal';
	import { getDeploymentTransactionArgs } from './getDeploymentTransactionArgs';
	import type { HandleAddOrderResult } from './getDeploymentTransactionArgs';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import { onMount } from 'svelte';
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import DepositInput from './DepositInput.svelte';
	import SelectToken from './SelectToken.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { useGui } from '$lib/hooks/useGui';

	interface Deployment {
		key: string;
		name: string;
		description: string;
	}

	export let settings: Writable<ConfigSource>;
	export let dotrain: string;
	export let deployment: Deployment;
	export let strategyDetail: NameAndDescriptionCfg;
	export let handleDeployModal: (args: DeployModalProps) => void;
	export let handleDisclaimerModal: (args: DisclaimerModalProps) => void;

	let allDepositFields: GuiDepositCfg[] = [];
	let allTokenOutputs: OrderIOCfg[] = [];
	let allFieldDefinitionsWithoutDefaults: GuiFieldDefinitionCfg[] = [];
	let allFieldDefinitionsWithDefaults: GuiFieldDefinitionCfg[] = [];
	let allTokensSelected: boolean = false;
	let showAdvancedOptions: boolean = false;
	let checkingDeployment: boolean = false;
	let allTokenInfos: TokenInfo[] = [];

	const gui = useGui();
	const selectTokens = gui.getSelectTokens();
	const networkKey = gui.getNetworkKey();
	const subgraphUrl = $settings?.subgraphs?.[networkKey] ?? '';

	let deploymentStepsError = DeploymentStepsError.error;

	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let signerAddress: Writable<string | null>;

	onMount(async () => {
		await areAllTokensSelected();
	});

	function getAllFieldDefinitions() {
		try {
			const allFieldDefinitionsResult = gui.getAllFieldDefinitions(false);
			if (allFieldDefinitionsResult.error) {
				throw new Error(allFieldDefinitionsResult.error.msg);
			}
			allFieldDefinitionsWithoutDefaults = allFieldDefinitionsResult.value;

			const allFieldDefinitionsWithDefaultsResult = gui.getAllFieldDefinitions(true);
			if (allFieldDefinitionsWithDefaultsResult.error) {
				throw new Error(allFieldDefinitionsWithDefaultsResult.error.msg);
			}
			allFieldDefinitionsWithDefaults = allFieldDefinitionsWithDefaultsResult.value;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_FIELD_DEFINITIONS);
		}
	}

	async function getAllDepositFields() {
		try {
			let result = gui.getCurrentDeployment();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			let depositFields = result.value.deposits;

			allDepositFields = depositFields;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_DEPOSITS);
		}
	}

	let allTokenInputs: OrderIOCfg[] = [];
	function getAllTokenInputs() {
		try {
			let result = gui.getCurrentDeployment();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			allTokenInputs = result.value.deployment.order.inputs;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_TOKEN_INPUTS);
		}
	}

	function getAllTokenOutputs() {
		try {
			let result = gui.getCurrentDeployment();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			allTokenOutputs = result.value.deployment.order.outputs;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_TOKEN_OUTPUTS);
		}
	}

	$: if (selectTokens?.length === 0 || allTokensSelected) {
		updateFields();
	}

	async function updateFields() {
		try {
			DeploymentStepsError.clear();

			getAllDepositFields();
			getAllFieldDefinitions();
			getAllTokenInputs();
			getAllTokenOutputs();
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_GUI);
		}
	}

	async function _handleShareChoices() {
		await handleShareChoices(gui);
	}

	async function onSelectTokenSelect() {
		await areAllTokensSelected();

		if (allTokensSelected) {
			let result = await gui.getAllTokenInfos();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			let newAllTokenInfos = result.value;
			if (allTokenInfos !== newAllTokenInfos) {
				allTokenInfos = newAllTokenInfos;
				getAllDepositFields();
				getAllFieldDefinitions();
			}
		}
	}

	async function handleDeployButtonClick() {
		DeploymentStepsError.clear();

		if (!allTokenOutputs) {
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_TOKEN_OUTPUTS);
			return;
		}
		if (!wagmiConfig) {
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_CHAIN);
			return;
		}

		if (!networkKey) {
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_CHAIN);
			return;
		}

		let result: HandleAddOrderResult | null = null;
		checkingDeployment = true;
		try {
			result = await getDeploymentTransactionArgs(gui, $wagmiConfig);
		} catch (e) {
			checkingDeployment = false;
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
		}
		if (!result) {
			checkingDeployment = false;
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
			return;
		}
		checkingDeployment = false;
		const onAccept = () => {
			if (!networkKey) {
				DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_CHAIN);
				return;
			}

			handleDeployModal({
				open: true,
				args: {
					...result,
					subgraphUrl: subgraphUrl,
					network: networkKey
				}
			});
		};

		handleDisclaimerModal({
			open: true,
			onAccept
		});
	}

	const areAllTokensSelected = async () => {
		try {
			allTokensSelected = gui.areAllTokensSelected();
			if (!allTokensSelected) return;

			let result = await gui.getAllTokenInfos();
			if (result.error) {
				throw new Error(result.error.msg);
			}
			allTokenInfos = result.value;

			// if we have deposits or vault ids set, show advanced options
			const hasDeposits = gui.hasAnyDeposit();
			const hasVaultIds = gui.hasAnyVaultId();
			if (hasDeposits || hasVaultIds) {
				showAdvancedOptions = true;
			}
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_SELECT_TOKENS);
		}
	};
</script>

<div>
	{#if $deploymentStepsError}
		<Alert color="red">
			<p class="text-red-500">{$deploymentStepsError.code}</p>
			{#if $deploymentStepsError.details}
				<p class="text-red-500">{$deploymentStepsError.details}</p>
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
					<div class="flex w-full flex-col gap-4">
						<DeploymentSectionHeader
							title="Select Tokens"
							description="Select the tokens that you want to use in your order."
						/>
						{#each selectTokens as token}
							<SelectToken {token} {onSelectTokenSelect} {gui} />
						{/each}
					</div>
				{/if}

				{#if allTokensSelected || selectTokens?.length === 0}
					{#if allFieldDefinitionsWithoutDefaults.length > 0}
						{#each allFieldDefinitionsWithoutDefaults as fieldDefinition}
							<FieldDefinitionInput {fieldDefinition} {gui} />
						{/each}
					{/if}

					<Toggle bind:checked={showAdvancedOptions}>Show advanced options</Toggle>

					{#if allFieldDefinitionsWithDefaults.length > 0 && showAdvancedOptions}
						{#each allFieldDefinitionsWithDefaults as fieldDefinition}
							<FieldDefinitionInput {fieldDefinition} {gui} />
						{/each}
					{/if}

					{#if showAdvancedOptions}
						{#each allDepositFields as deposit}
							<DepositInput {deposit} {gui} />
						{/each}
					{/if}

					{#if showAdvancedOptions}
						{#each allTokenInputs as input, i}
							<TokenIOInput {i} label="Input" vault={input} {gui} />
						{/each}

						{#each allTokenOutputs as output, i}
							<TokenIOInput {i} label="Output" vault={output} {gui} />
						{/each}
					{/if}

					{#if $deploymentStepsError}
						<Alert color="red">
							<p class="text-red-500">{$deploymentStepsError.code}</p>
							{#if $deploymentStepsError.details}
								<p class="text-red-500">{$deploymentStepsError.details}</p>
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
							<WalletConnect {appKitModal} connected={wagmiConnected} {signerAddress} />
						{/if}
						<ComposedRainlangModal {gui} />
						<ShareChoicesButton handleShareChoices={_handleShareChoices} />
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</div>
