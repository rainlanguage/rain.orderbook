<script lang="ts">
	import FieldDefinitionButtons from './FieldDefinitionButtons.svelte';
	import DepositButtons from './DepositButtons.svelte';
	import SelectToken from '../SelectToken.svelte';
	import TokenInputButtons from './TokenInputButtons.svelte';
	import TokenOutputButtons from './TokenOutputButtons.svelte';

	import type {
		DotrainOrderGui,
		GuiDeposit,
		GuiFieldDefinition,
		SelectTokens,
		TokenInfos,
		Vault
	} from '@rainlanguage/orderbook/js_api';
	import { Button } from 'flowbite-svelte';
	import { getDeploymentSteps } from './getDeploymentSteps';
	import deploymentStore from './deploymentStore';
	export let gui: DotrainOrderGui;

	export let selectTokens: SelectTokens;
	export let allFieldDefinitions: GuiFieldDefinition[];
	export let allTokenInputs: Vault[];
	export let allTokenOutputs: Vault[];
	export let allDeposits: GuiDeposit[];
	export let inputVaultIds: string[];
	export let outputVaultIds: string[];
	export let isLimitStrat: boolean;
	export let useCustomVaultIds: boolean;
	export let handleAddOrder: () => Promise<void>;
	export let tokenInfos: TokenInfos;

	let deploymentSteps = getDeploymentSteps(
		selectTokens,
		isLimitStrat,
		allFieldDefinitions,
		gui,
		allDeposits,
		allTokenInputs,
		allTokenOutputs,
		inputVaultIds,
		outputVaultIds,
		tokenInfos
	);

	deploymentStore.populateDeploymentSteps(deploymentSteps);

	$: console.log('DEPLOYMENT STEPS', $deploymentStore.deploymentSteps);

	let currentStep = deploymentSteps[0];

	const nextStep = () => {
		if (deploymentSteps.indexOf(currentStep) < deploymentSteps.length - 1)
			currentStep = deploymentSteps[deploymentSteps.indexOf(currentStep) + 1];
	};

	const previousStep = () => {
		if (deploymentSteps.indexOf(currentStep) > 0)
			currentStep = deploymentSteps[deploymentSteps.indexOf(currentStep) - 1];
	};
</script>

<div class="flex h-[80vh] flex-col justify-between">
	<div class="text-lg text-gray-800 dark:text-gray-200">
		Step {deploymentSteps.indexOf(currentStep) + 1} of {deploymentSteps.length}
	</div>

	{#if currentStep.type === 'tokens'}
		<SelectToken {...currentStep} />
	{:else if currentStep.type === 'fields'}
		<FieldDefinitionButtons {...currentStep} />
	{:else if currentStep.type === 'deposits'}
		<DepositButtons {...currentStep} />
	{:else if currentStep.type === 'tokenInput'}
		<TokenInputButtons {...currentStep} />
	{:else if currentStep.type === 'tokenOutput'}
		<TokenOutputButtons {...currentStep} />
	{/if}

	<div class="flex justify-between gap-4">
		{#if deploymentSteps.indexOf(currentStep) > 0}
			<Button class="flex-1" on:click={previousStep}>Previous</Button>
		{/if}

		{#if deploymentSteps.indexOf(currentStep) === deploymentSteps.length - 1}
			<Button class="flex-1" on:click={handleAddOrder}>Add Order</Button>
		{:else}
			<Button
				class="flex-1"
				on:click={nextStep}
				disabled={deploymentSteps.indexOf(currentStep) === deploymentSteps.length - 1}
			>
				Next
			</Button>
		{/if}
	</div>
</div>
