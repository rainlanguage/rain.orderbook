<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let token: string;
	export let gui: DotrainOrderGui;
	export let selectTokens: Map<string, string>;
</script>

<div class="mb-4 flex flex-col gap-2">
	<Label class="whitespace-nowrap text-xl">{token}</Label>
	<Input
		type="text"
		on:change={async ({ currentTarget }) => {
			if (currentTarget instanceof HTMLInputElement) {
				if (!gui) return;
				await gui.saveSelectTokenAddress(token, currentTarget.value);
				selectTokens = gui.getSelectTokens();
				gui = gui;
			}
		}}
	/>
</div>
