<script lang="ts">
	import FieldDefinitionButtons from './FieldDefinitionButtons.svelte';
	import DepositButtons from './DepositButtons.svelte';
	import SelectToken from '../SelectToken.svelte';
	import TokenInputButtons from './TokenInputButtons.svelte';
	import TokenOutputButtons from './TokenOutputButtons.svelte';
	import type { WizardStep } from '../../../types/wizardSteps';

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

	$: if (currentStep) {
		const fieldValues = gui.getAllFieldValues();
		console.log(fieldValues);
	}

	type TokenProps = {
		token: string;
		gui: DotrainOrderGui;
		selectTokens: SelectTokens;
	};

	type FieldProps = {
		fieldDefinition: GuiFieldDefinition;
		gui: DotrainOrderGui;
	};

	type DepositProps = {
		deposit: GuiDeposit;
		gui: DotrainOrderGui;
		tokenInfos: TokenInfos;
	};

	type TokenInputProps = {
		i: number;
		input: Vault;
		tokenInfos: TokenInfos;
		inputVaultIds: string[];
		gui: DotrainOrderGui;
	};

	type TokenOutputProps = {
		i: number;
		output: Vault;
		tokenInfos: TokenInfos;
		outputVaultIds: string[];
		gui: DotrainOrderGui;
	};

	type WizardStep<T> = {
		type: 'tokens' | 'fields' | 'deposits' | 'tokenInput' | 'tokenOutput';
		data: T;
	};

	let steps: (
		| WizardStep<TokenProps>
		| WizardStep<FieldProps>
		| WizardStep<DepositProps>
		| WizardStep<TokenInputProps>
		| WizardStep<TokenOutputProps>
	)[] = [
		...(selectTokens.size > 0 && isLimitStrat
			? Array.from(selectTokens.entries()).map(
					([token]): WizardStep<TokenProps> => ({
						type: 'tokens',
						data: { token, gui, selectTokens } as {
							token: string;
							gui: DotrainOrderGui;
							selectTokens: SelectTokens;
						}
					})
				)
			: []),

		...allFieldDefinitions.map(
			(fieldDefinition): WizardStep<FieldProps> => ({
				type: 'fields',
				data: { fieldDefinition, gui } as {
					fieldDefinition: GuiFieldDefinition;
					gui: DotrainOrderGui;
				}
			})
		),

		...allDeposits.map(
			(deposit): WizardStep<DepositProps> => ({
				type: 'deposits',
				data: { deposit, gui, tokenInfos } as {
					deposit: GuiDeposit;
					gui: DotrainOrderGui;
					tokenInfos: TokenInfos;
				}
			})
		),

		...allTokenInputs.map(
			(input, i): WizardStep<TokenInputProps> => ({
				type: 'tokenInput',
				data: { input, gui, tokenInfos, i, inputVaultIds } as {
					input: Vault;
					gui: DotrainOrderGui;
					tokenInfos: TokenInfos;
					i: number;
					inputVaultIds: string[];
				}
			})
		),
		...allTokenOutputs.map(
			(output, i): WizardStep<TokenOutputProps> => ({
				type: 'tokenOutput',
				data: { output, gui, tokenInfos, i, outputVaultIds } as {
					output: Vault;
					gui: DotrainOrderGui;
					tokenInfos: TokenInfos;
					i: number;
					outputVaultIds: string[];
				}
			})
		)
	];

	let currentStep = 0;

	const nextStep = () => {
		if (currentStep < totalSteps - 1) currentStep++;
	};

	const previousStep = () => {
		if (currentStep > 0) currentStep--;
	};
</script>

<div class="flex h-[80vh] flex-col justify-between">
	<div class="text-lg text-gray-800 dark:text-gray-200">
		Step {currentStep + 1} of {totalSteps}
	</div>

	{#if steps[currentStep].type === 'tokens'}
		<SelectToken {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'fields'}
		<FieldDefinitionButtons {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'deposits'}
		<DepositButtons {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'tokenInput'}
		<TokenInputButtons {...steps[currentStep].data} />
	{:else if steps[currentStep].type === 'tokenOutput'}
		<TokenOutputButtons {...steps[currentStep].data} />
	{/if}

	<div class="flex justify-between gap-4">
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
