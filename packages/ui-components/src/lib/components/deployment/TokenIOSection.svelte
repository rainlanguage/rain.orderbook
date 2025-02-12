<script lang="ts">
	import TokenIOInput from './TokenIOInput.svelte';
	import type { DotrainOrderGui, OrderIO } from '@rainlanguage/orderbook/js_api';

	export let allTokenInputs: OrderIO[] = [];
	export let allTokenOutputs: OrderIO[] = [];
	export let gui: DotrainOrderGui;
	export let inputVaultIds: string[] = [];
	export let outputVaultIds: string[] = [];
	export let open: boolean;
</script>

{#if allTokenInputs.length > 0}
	{#each allTokenInputs as input, i}
		<TokenIOInput
			isInput={true}
			{i}
			label="Input"
			vault={input}
			vaultIds={inputVaultIds}
			{gui}
			{open}
		/>
	{/each}
{/if}

{#if allTokenOutputs.length > 0}
	{#each allTokenOutputs as output, i}
		<TokenIOInput
			isInput={false}
			{i}
			label="Output"
			vault={output}
			vaultIds={outputVaultIds}
			{gui}
			{open}
		/>
	{/each}
{/if}
