<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { TokenInfos, Vault } from '@rainlanguage/orderbook/js_api';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let i: number;
	export let input: Vault;
	export let tokenInfos: TokenInfos;
	export let inputVaultIds: string[];
	export let gui: DotrainOrderGui;
</script>

<div class="flex items-center gap-2">
	<Label class="whitespace-nowrap"
		>Input {i + 1} ({tokenInfos.get(input.token.address)?.symbol})</Label
	>
	<Input
		type="text"
		placeholder="Enter vault ID"
		bind:value={inputVaultIds[i]}
		on:change={() => gui?.setVaultId(true, i, inputVaultIds[i])}
	/>
</div>
