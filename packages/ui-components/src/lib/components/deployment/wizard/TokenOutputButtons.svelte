<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { TokenInfos, Vault, DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import type { StepType } from '../../../types/wizardSteps';

	export let i: number;
	export let output: Vault;
	export let tokenInfos: TokenInfos;
	export let outputVaultIds: string[];
	export let gui: DotrainOrderGui;
	export let type: StepType;
</script>

<div class="flex items-center gap-2">
	<Label class="whitespace-nowrap"
		>Output {i + 1} ({tokenInfos.get(output.token.address)?.symbol})</Label
	>
	<Input
		type="text"
		placeholder="Enter vault ID"
		bind:value={outputVaultIds[i]}
		on:change={() => gui?.setVaultId(false, i, outputVaultIds[i])}
	/>
</div>
