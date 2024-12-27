<script lang="ts">
	import FieldDefinitionButtons from './FieldDefinitionButtons.svelte';
	import DepositButtons from './DepositButtons.svelte';
	import SelectToken from '../SelectToken.svelte';
	import TokenInputButtons from './TokenInputButtons.svelte';
	import TokenOutputButtons from './TokenOutputButtons.svelte';

	import type {
		DotrainOrderGui,
		TokenDeposit,
		GuiFieldDefinition,
		SelectTokens,
		TokenInfos,
		Vault
	} from '@rainlanguage/orderbook/js_api';
	import { Button } from 'flowbite-svelte';
	import { getDeploymentSteps } from './getDeploymentSteps';
	import deploymentStepsStore from './deploymentStepsStore';
	export let gui: DotrainOrderGui;

	export let selectTokens: SelectTokens;
	export let allFieldDefinitions: GuiFieldDefinition[];
	export let allTokenInputs: Vault[];
	export let allTokenOutputs: Vault[];
	export let allDeposits: TokenDeposit[];
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

	deploymentStepsStore.populateDeploymentSteps(deploymentSteps);

	$: currentStep = 0;

	const nextStep = () => {
		if (currentStep < deploymentSteps.length - 1) {
			currentStep++;
		}
	};

	const previousStep = () => {
		if (currentStep > 0) {
			currentStep--;
		}
	};
</script>

<div class="flex h-[80vh] flex-col justify-between">
	<div class="text-lg text-gray-800 dark:text-gray-200">
		Step {currentStep + 1} of {deploymentSteps.length}
	</div>

	{#if deploymentSteps[currentStep].type === 'tokens'}
		<SelectToken {...deploymentSteps[currentStep]} />
	{:else if deploymentSteps[currentStep].type === 'fields'}
		<FieldDefinitionButtons {...deploymentSteps[currentStep]} {currentStep} />
	{:else if deploymentSteps[currentStep].type === 'deposits'}
		<DepositButtons {...deploymentSteps[currentStep]} />
	{:else if deploymentSteps[currentStep].type === 'tokenInput'}
		<TokenInputButtons {...deploymentSteps[currentStep]} {useCustomVaultIds} />
	{:else if deploymentSteps[currentStep].type === 'tokenOutput'}
		<TokenOutputButtons {...deploymentSteps[currentStep]} {useCustomVaultIds} />
	{/if}

	<div class="flex justify-between gap-4">
		{#if currentStep > 0}
			<Button class="flex-1" on:click={previousStep}>Previous</Button>
		{/if}

		{#if currentStep === deploymentSteps.length - 1}
			<Button class="flex-1" on:click={handleAddOrder}>Add Order</Button>
		{:else}
			<Button
				class="flex-1"
				on:click={nextStep}
				disabled={currentStep === deploymentSteps.length - 1}
			>
				Next
			</Button>
		{/if}
	</div>
</div>
