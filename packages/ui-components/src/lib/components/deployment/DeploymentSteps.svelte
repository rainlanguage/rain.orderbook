<script lang="ts">
	import FieldDefinitionButtons from './FieldDefinitionButtons.svelte';
	import DepositButtons from './DepositButtons.svelte';
	import SelectToken from './SelectToken.svelte';
	import TokenInput from './TokenInput.svelte';
	import TokenOutput from './TokenOutput.svelte';
	import type {
		DotrainOrderGui,
		GuiDeposit,
		GuiFieldDefinition,
		SelectTokens,
		TokenInfos,
		Vault
	} from '@rainlanguage/orderbook/js_api';
	import { Button } from 'flowbite-svelte';
	export let gui: DotrainOrderGui;

	export let selectTokens: SelectTokens;
	export let allFieldDefinitions: GuiFieldDefinition[];
	export let allTokenInputs: Vault[];
	export let allTokenOutputs: Vault[];
	export let allDeposits: GuiDeposit[];
	export let inputVaultIds: string[];
	export let outputVaultIds: string[];

	$: totalSteps =
		(selectTokens?.size || 0) +
		(allFieldDefinitions?.length || 0) +
		(allFieldDefinitions?.length || 0) +
		(allDeposits?.length || 0) +
		(useCustomVaultIds && (allTokenInputs.length > 0 || allTokenOutputs.length > 0) ? 1 : 0);

	export let isLimitStrat: boolean;
	export let useCustomVaultIds: boolean;
	export let handleAddOrder: () => Promise<void>;
	export let tokenInfos: TokenInfos;

	$: steps = [
		...(selectTokens.size > 0 && isLimitStrat
			? Array.from(selectTokens.entries()).map(([token]) => ({
					type: 'tokens' as const,
					data: { token, gui, selectTokens }
				}))
			: []),

		...allFieldDefinitions.map((fieldDefinition) => ({
			type: 'fields' as const,
			data: { fieldDefinition, gui }
		})),

		...allDeposits.map((deposit) => ({
			type: 'deposits' as const,
			data: { deposit, gui, tokenInfos }
		})),

		...allTokenInputs.map((input, i) => ({
			type: 'tokenInput' as const,
			data: { input, gui, tokenInfos, i, inputVaultIds }
		})),
		...allTokenOutputs.map((output, i) => ({
			type: 'tokenOutput' as const,
			data: { output, gui, tokenInfos, i, outputVaultIds }
		}))
	];

	let currentStep = 0;

	const nextStep = () => {
		if (currentStep < totalSteps - 1) currentStep++;
	};

	const previousStep = () => {
		if (currentStep > 0) currentStep--;
	};
</script>

<div class="flex flex-col gap-4">
	<!-- Show current progress -->
	<div class="text-sm text-gray-600">
		Step {currentStep + 1} of {totalSteps}
	</div>

	<!-- Content sections -->
	{#if steps[currentStep].type === 'tokens'}
		<SelectToken {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'fields'}
		<FieldDefinitionButtons {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'deposits'}
		<DepositButtons {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'tokenInput'}
		<TokenInput {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'tokenOutput'}
		<TokenOutput {...steps[currentStep].data} />
	{/if}

	<!-- Navigation buttons -->
	<div class="mt-4 flex justify-between gap-4">
		<Button class="flex-1" on:click={previousStep} disabled={currentStep === 0}>Previous</Button>

		{#if currentStep === totalSteps - 1}
			<Button class="flex-1" on:click={handleAddOrder}>Add Order</Button>
		{:else}
			<Button class="flex-1" on:click={nextStep} disabled={currentStep === totalSteps - 1}>
				Next
			</Button>
		{/if}
	</div>
</div>
