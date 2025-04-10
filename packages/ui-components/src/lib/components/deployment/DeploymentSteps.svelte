<script lang="ts">
	import { Alert, Button, Spinner, Toggle } from 'flowbite-svelte';
	import TokenIOInput from './TokenIOInput.svelte';
	import ComposedRainlangModal from './ComposedRainlangModal.svelte';
	import {
		type GuiSelectTokensCfg,
		type TokenInfo,
		type GuiDepositCfg,
		type GuiFieldDefinitionCfg,
		type NameAndDescriptionCfg,
		type OrderIOCfg,
		OrderbookYaml
	} from '@rainlanguage/orderbook';
	import WalletConnect from '../wallet/WalletConnect.svelte';
	import { type Writable } from 'svelte/store';
	import type { AppKit } from '@reown/appkit';
	import { handleShareChoices } from '../../services/handleShareChoices';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import { onMount } from 'svelte';
	import FieldDefinitionInput from './FieldDefinitionInput.svelte';
	import DepositInput from './DepositInput.svelte';
	import SelectToken from './SelectToken.svelte';
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import { useGui } from '$lib/hooks/useGui';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { handleDeployment } from './handleDeployment';
	import { type DeploymentArgs } from '$lib/types/transaction';
	import { fade } from 'svelte/transition';
	import ShareChoicesButton from './ShareChoicesButton.svelte';

	interface Deployment {
		key: string;
		name: string;
		description: string;
	}

	/** The deployment configuration containing key, name and description */
	export let deployment: Deployment;
	/** Strategy details containing name and description configuration */
	export let strategyDetail: NameAndDescriptionCfg;
	/** Handlers for deployment modals */
	export let onDeploy: (deploymentArgs: DeploymentArgs) => void;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let registryUrl: string;

	let allDepositFields: GuiDepositCfg[] = [];
	let allTokenOutputs: OrderIOCfg[] = [];
	let allFieldDefinitionsWithoutDefaults: GuiFieldDefinitionCfg[] = [];
	let allFieldDefinitionsWithDefaults: GuiFieldDefinitionCfg[] = [];
	let allTokensSelected: boolean = false;
	let showAdvancedOptions: boolean = false;
	let allTokenInfos: TokenInfo[] = [];
	let selectTokens: GuiSelectTokensCfg[] | undefined = undefined;
	let checkingDeployment: boolean = false;
	let subgraphUrl: string = '';

	const { account } = useAccount();
	const gui = useGui();

	let deploymentStepsError = DeploymentStepsError.error;

	onMount(async () => {
		const selectTokensResult = gui.getSelectTokens();
		if (selectTokensResult.error) {
			throw new Error(selectTokensResult.error.msg);
		}
		selectTokens = selectTokensResult.value;

		const dotrainResult = gui.generateDotrainText();
		if (dotrainResult.error) {
			throw new Error(dotrainResult.error.msg);
		}

		const orderbookYaml = new OrderbookYaml([dotrainResult.value]);
		const orderbook = orderbookYaml.getOrderbookByDeploymentKey(deployment.key);
		if (orderbook.error) {
			throw new Error(orderbook.error.msg);
		}
		subgraphUrl = orderbook.value.subgraph.url;

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
		await handleShareChoices(gui, registryUrl);
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

	const areAllTokensSelected = async () => {
		try {
			const areAllTokensSelectedResult = gui.areAllTokensSelected();
			if (areAllTokensSelectedResult.error) {
				throw new Error(areAllTokensSelectedResult.error.msg);
			}
			allTokensSelected = areAllTokensSelectedResult.value;
			if (!allTokensSelected) return;

			const getAllTokenInfosResult = await gui.getAllTokenInfos();
			if (getAllTokenInfosResult.error) {
				throw new Error(getAllTokenInfosResult.error.msg);
			}
			allTokenInfos = getAllTokenInfosResult.value;

			// if we have deposits or vault ids set, show advanced options
			const hasDepositsResult = gui.hasAnyDeposit();
			if (hasDepositsResult.error) {
				throw new Error(hasDepositsResult.error.msg);
			}
			const hasVaultIdsResult = gui.hasAnyVaultId();
			if (hasVaultIdsResult.error) {
				throw new Error(hasVaultIdsResult.error.msg);
			}
			if (hasDepositsResult.value || hasVaultIdsResult.value) {
				showAdvancedOptions = true;
			}
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_SELECT_TOKENS);
		}
	};

	async function handleDeployButtonClick() {
		if (checkingDeployment) {
			return;
		}
		checkingDeployment = true;
		try {
			if (!$account) {
				DeploymentStepsError.catch(
					'No wallet connected',
					DeploymentStepsErrorCode.ADD_ORDER_FAILED
				);
				return;
			}
			DeploymentStepsError.clear();
			const deploymentArgs: DeploymentArgs = await handleDeployment(gui, $account, subgraphUrl);
			return await onDeploy(deploymentArgs);
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
		} finally {
			checkingDeployment = false;
		}
	}
</script>

<div>
	{#if gui}
		<div class="flex max-w-3xl flex-col gap-12" in:fade>
			{#if deployment}
				<div class="flex max-w-2xl flex-col gap-4 text-start">
					<h1 class="text-4xl font-semibold text-gray-900 lg:text-6xl dark:text-white">
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
						<SelectToken {token} {onSelectTokenSelect} />
					{/each}
				</div>
			{/if}

			{#if allTokensSelected || selectTokens?.length === 0}
				{#if allFieldDefinitionsWithoutDefaults.length > 0}
					{#each allFieldDefinitionsWithoutDefaults as fieldDefinition}
						<FieldDefinitionInput {fieldDefinition} />
					{/each}
				{/if}

				<Toggle bind:checked={showAdvancedOptions}>Show advanced options</Toggle>

				{#if showAdvancedOptions}
					{#each allFieldDefinitionsWithDefaults as fieldDefinition}
						<FieldDefinitionInput {fieldDefinition} />
					{/each}

					{#each allDepositFields as deposit}
						<DepositInput {deposit} />
					{/each}

					{#each allTokenInputs as input, i}
						<TokenIOInput {i} label="Input" vault={input} />
					{/each}

					{#each allTokenOutputs as output, i}
						<TokenIOInput {i} label="Output" vault={output} />
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
					{#if $account}
						<Button
							data-testid="deploy-button"
							size="lg"
							disabled={checkingDeployment}
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
					<ComposedRainlangModal />
					<ShareChoicesButton handleShareChoices={_handleShareChoices} />
				</div>
			{/if}
		</div>
	{/if}
</div>
