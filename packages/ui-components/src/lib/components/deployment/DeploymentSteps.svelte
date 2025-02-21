<script lang="ts">
	import { Alert } from 'flowbite-svelte';
	import TokenIOSection from './TokenIOSection.svelte';
	import DepositsSection from './DepositsSection.svelte';
	import SelectTokensSection from './SelectTokensSection.svelte';
	import ComposedRainlangModal from './ComposedRainlangModal.svelte';
	import FieldDefinitionsSection from './FieldDefinitionsSection.svelte';
	import { type ConfigSource } from '@rainlanguage/orderbook/js_api';
	import WalletConnect from '../wallet/WalletConnect.svelte';
	import {
		DotrainOrderGui,
		type GuiDepositCfg,
		type GuiFieldDefinitionCfg,
		type NameAndDescriptionCfg,
		type GuiDeploymentCfg,
		type OrderIOCfg,
		type SelectTokens,
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
	import { handleShareChoices } from '../../services/handleShareChoices';
	import type { DisclaimerModalProps, DeployModalProps } from '../../types/modal';
	import { getDeploymentTransactionArgs } from './getDeploymentTransactionArgs';
	import type { HandleAddOrderResult } from './getDeploymentTransactionArgs';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';

	export let settings: Writable<ConfigSource>;
	export let dotrain: string;
	export let deployment: GuiDeploymentCfg;
	export let strategyDetail: NameAndDescriptionCfg;

	export let handleDeployModal: (args: DeployModalProps) => void;
	export let handleDisclaimerModal: (args: DisclaimerModalProps) => void;
	export let pushGuiStateToUrlHistory: (serializedState: string) => void;

	let selectTokens: SelectTokens | null = null;
	let allDepositFields: GuiDepositCfg[] = [];
	let allTokenOutputs: OrderIOCfg[] = [];
	let allFieldDefinitions: GuiFieldDefinitionCfg[] = [];
	let allTokensSelected: boolean = false;
	let showAdvancedOptions: boolean = false;
	let gui: DotrainOrderGui | null = null;
	let checkingDeployment: boolean = false;
	let networkKey: string | null = null;
	let subgraphUrl: string = '';
	let allTokenInfos: AllTokenInfos = [];

	let deploymentStepsError = DeploymentStepsError.error;

	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let stateFromUrl: string | null = null;
	$: if (deployment) {
		handleDeploymentChange(deployment.key);
	}

	async function handleDeploymentChange(deployment: string) {
		if (!deployment || !dotrain) return;
		DeploymentStepsError.clear();

		try {
			gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment, pushGuiStateToUrlHistory);

			if (gui) {
				networkKey = await gui.getNetworkKey();
				subgraphUrl = $settings?.subgraphs?.[networkKey] ?? '';
				try {
					selectTokens = gui.getSelectTokens();
					return selectTokens;
				} catch (e) {
					DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_SELECT_TOKENS);
				}
			}
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_GUI);
		}
	}

	function getAllFieldDefinitions() {
		if (!gui) return;
		try {
			allFieldDefinitions = gui.getAllFieldDefinitions();
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_FIELD_DEFINITIONS);
		}
	}

	async function getAllDepositFields() {
		if (!gui) return;
		try {
			let dep: GuiDeploymentCfg = gui.getCurrentDeployment();
			let depositFields: GuiDepositCfg[] = dep.deposits;

			allDepositFields = depositFields;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_DEPOSITS);
		}
	}

	let allTokenInputs: OrderIOCfg[] = [];
	function getAllTokenInputs() {
		if (!gui) return;

		try {
			allTokenInputs = gui.getCurrentDeployment().deployment.order.inputs;
		} catch (e) {
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_TOKEN_INPUTS);
		}
	}

	function getAllTokenOutputs() {
		if (!gui) return;
		try {
			allTokenOutputs = gui.getCurrentDeployment().deployment.order.outputs;
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
			$page.url.searchParams.get('state') || '',
			pushGuiStateToUrlHistory
		);
		areAllTokensSelected();
	}

	async function onSelectTokenSelect() {
		if (!gui) return;

		await areAllTokensSelected();

		if (allTokensSelected) {
			let newAllTokenInfos = await gui.getAllTokenInfos();
			if (allTokenInfos !== newAllTokenInfos) {
				allTokenInfos = newAllTokenInfos;
				getAllDepositFields();
				getAllFieldDefinitions();
			}
		}
	}

	async function handleDeployButtonClick() {
		DeploymentStepsError.clear();

		if (!gui) {
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_GUI);
			return;
		}
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
				DeploymentStepsError.catch(e, DeploymentStepsErrorCode.NO_SELECT_TOKENS);
			}
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
					<SelectTokensSection {gui} {selectTokens} {onSelectTokenSelect} />
				{/if}

				{#if allTokensSelected || selectTokens?.length === 0}
					{#if allFieldDefinitions.length > 0}
						<FieldDefinitionsSection {allFieldDefinitions} {gui} />
					{/if}

					<Toggle bind:checked={showAdvancedOptions}>Show advanced options</Toggle>

					{#if allDepositFields.length > 0 && showAdvancedOptions}
						<DepositsSection bind:allDepositFields {gui} />
					{/if}

					{#if allTokenInputs.length > 0 && allTokenOutputs.length > 0 && showAdvancedOptions}
						<TokenIOSection bind:allTokenInputs bind:allTokenOutputs {gui} />
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
