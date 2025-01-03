<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import type { TokenInfos, Vault } from '@rainlanguage/orderbook/js_api';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let i: number;
	export let label: 'Input' | 'Output';
	export let vault: Vault;
	export let tokenInfos: TokenInfos;
	export let vaultIds: string[];
	export let gui: DotrainOrderGui;
</script>

<div class="flex flex-grow flex-col items-center p-8">
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">
			{label}
			{i + 1} ({tokenInfos.get(vault.token.address)?.symbol || 'Unknown'})
		</h1>
	</div>
	<div class="mt-8 w-full max-w-md">
		<Input
			class="text-center text-lg"
			size="lg"
			type="text"
			placeholder="Enter vault ID"
			bind:value={vaultIds[i]}
			on:change={() => gui?.setVaultId(true, i, vaultIds[i])}
		/>
	</div>
</div>
