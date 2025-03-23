<script lang="ts">
	import TokenIOInput from './TokenIOInput.svelte';
	import type { DotrainOrderGui, OrderIOCfg } from '@rainlanguage/orderbook';

	export let allTokenInputs: OrderIOCfg[] = [];
	export let allTokenOutputs: OrderIOCfg[] = [];
	export let gui: DotrainOrderGui;
</script>

{#if allTokenInputs.length > 0}
	{#each allTokenInputs as input, i}
		<TokenIOInput {i} label="Input" vault={input} {gui} />
	{/each}
{/if}

{#if allTokenOutputs.length > 0}
	{#each allTokenOutputs as output, i}
		<TokenIOInput {i} label="Output" vault={output} {gui} />
	{/each}
{/if}
