<script lang="ts">
	import TokenIOInput from './TokenIOInput.svelte';
	import type { DotrainOrderGui, OrderIO } from '@rainlanguage/orderbook/js_api';

	export let allTokenInputs: OrderIO[] = [];
	export let allTokenOutputs: OrderIO[] = [];
	export let gui: DotrainOrderGui;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;
</script>

{#if allTokenInputs.length > 0}
	{#each allTokenInputs as input, i}
		<TokenIOInput {i} label="Input" vault={input} {gui} {handleUpdateGuiState} />
	{/each}
{/if}

{#if allTokenOutputs.length > 0}
	{#each allTokenOutputs as output, i}
		<TokenIOInput {i} label="Output" vault={output} {gui} {handleUpdateGuiState} />
	{/each}
{/if}
