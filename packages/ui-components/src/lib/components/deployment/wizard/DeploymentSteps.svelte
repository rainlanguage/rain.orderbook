<script lang="ts">
	import FieldDefinitionButtons from './FieldDefinitionButtons.svelte';
	import DepositButtons from './DepositButtons.svelte';
	import SelectToken from '../SelectToken.svelte';
	import TokenInputOrOutputWizard from './TokenInputOrOutputWizard.svelte';

	import type {
		DotrainOrderGui,
		GuiDeposit,
		GuiFieldDefinition,
		SelectTokens,
		TokenInfos,
		Vault
	} from '@rainlanguage/orderbook/js_api';
	import { Button, Label } from 'flowbite-svelte';
	export let gui: DotrainOrderGui;

	export let selectTokens: SelectTokens;
	export let allFieldDefinitions: GuiFieldDefinition[];
	export let allTokenInputs: Vault[];
	export let allTokenOutputs: Vault[];
	export let allDeposits: GuiDeposit[];
	export let inputVaultIds: string[];
	export let outputVaultIds: string[];
	export let isLimitStrat: boolean;
	export let handleAddOrder: () => Promise<void>;
	export let tokenInfos: TokenInfos;
</script>

<div class="flex h-[80vh] flex-col justify-between">
	{#if isLimitStrat && selectTokens.size > 0}
		<Label class="my-4 whitespace-nowrap text-2xl underline">Select Tokens</Label>

		{#each selectTokens.entries() as [token]}
			<SelectToken {token} {gui} {selectTokens} />
		{/each}
	{/if}

	{#if allFieldDefinitions.length > 0}
		<Label class="my-4 whitespace-nowrap text-2xl underline">Field Values</Label>
		{#each allFieldDefinitions as fieldDefinition}
			<FieldDefinitionButtons {fieldDefinition} {gui} />
		{/each}
	{/if}

	{#if allDeposits.length > 0}
		<Label class="my-4 whitespace-nowrap text-2xl underline">Deposits</Label>
		{#each allDeposits as deposit}
			<DepositButtons {deposit} {gui} {tokenInfos} />
		{/each}
	{/if}

	{#if allTokenInputs.length > 0}
		<Label class="whitespace-nowrap text-xl">Input Vault IDs</Label>
		{#each allTokenInputs as input, i}
			<TokenInputOrOutputWizard
				{i}
				label="Input"
				vault={input}
				{tokenInfos}
				vaultIds={inputVaultIds}
				{gui}
			/>
		{/each}
	{/if}

	{#if allTokenOutputs.length > 0}
		<Label class="whitespace-nowrap text-xl">Output Vault IDs</Label>
		{#each allTokenOutputs as output, i}
			<TokenInputOrOutputWizard
				{i}
				label="Output"
				vault={output}
				{tokenInfos}
				vaultIds={outputVaultIds}
				{gui}
			/>
		{/each}
	{/if}
	<Button class="flex-1" on:click={handleAddOrder}>Add Order</Button>
</div>
