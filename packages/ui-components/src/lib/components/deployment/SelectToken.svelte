<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let token: [string, string];
	export let gui: DotrainOrderGui;
	export let selectTokens: Map<string, string>;
	let inputValue: string | null = null;

	let error = '';

	async function handleInput(event: Event) {
		const currentTarget = event.currentTarget;
		if (currentTarget instanceof HTMLInputElement) {
			inputValue = currentTarget.value;
			if (!gui) return;
			try {
				await gui.saveSelectTokenAddress(token[0], currentTarget.value);
				error = '';
				selectTokens = gui.getSelectTokens();
				gui = gui;
			} catch {
				error = 'Invalid address';
			}
		}
	}

	$: if (token && !inputValue && inputValue !== '') {
		inputValue = token[1] || '';
	}
</script>

<div class="mb-4 flex flex-col gap-2">
	<Label class="whitespace-nowrap text-xl">{token[0]}</Label>
	<Input type="text" on:input={handleInput} bind:value={inputValue} />
	{#if error}
		<p class="text-red-500">{error}</p>
	{/if}
</div>
