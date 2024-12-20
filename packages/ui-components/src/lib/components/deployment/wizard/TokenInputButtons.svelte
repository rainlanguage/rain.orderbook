<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import type { TokenInfos, Vault, DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let i: number;
	export let input: Vault;
	export let tokenInfos: TokenInfos;
	export let inputVaultIds: string[];
	export let gui: DotrainOrderGui;
</script>

<div class="flex flex-grow flex-col items-center">
	<div class="mt-16 max-w-2xl text-center">
		<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">
			Input {i + 1} ({tokenInfos.get(input.token.address)?.symbol})
		</h1>
		<p class="mb-12 text-xl text-gray-600">Enter vault ID</p>
	</div>

	<div class="mt-8 w-full max-w-md">
		<Input
			class="text-center text-lg"
			size="lg"
			placeholder="Enter vault ID"
			bind:value={inputVaultIds[i]}
			on:change={() => gui?.setVaultId(true, i, inputVaultIds[i])}
		/>
	</div>
</div>
