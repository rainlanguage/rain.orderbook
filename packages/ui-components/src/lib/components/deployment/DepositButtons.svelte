<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenInfos
	} from '@rainlanguage/orderbook/js_api';
	import { Label, Input, Button } from 'flowbite-svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;
	export let tokenInfos: TokenInfos;

	let showCustomInput = false;

	function handlePresetClick(preset: string) {
		gui?.saveDeposit(deposit.token_name, preset);
		showCustomInput = false;
		gui = gui;
	}

	function handleCustomClick() {
		showCustomInput = true;
		gui?.saveDeposit(deposit.token_name, '');
		gui = gui;
	}
</script>

<div class="mb-4 flex flex-col gap-2">
	<Label class="whitespace-nowrap text-xl">{tokenInfos.get(deposit.token.address)?.name}</Label>

	<div class="flex flex-wrap gap-2">
		{#each deposit.presets as preset}
			<Button
				color={gui?.isDepositPreset(deposit.token_name) &&
				gui?.getDeposit(deposit.token_name) === preset
					? 'blue'
					: 'alternative'}
				size="sm"
				on:click={() => handlePresetClick(preset)}
			>
				{preset}
			</Button>
		{/each}
		<Button
			color={!gui?.isDepositPreset(deposit.token_name) ? 'blue' : 'alternative'}
			size="sm"
			on:click={handleCustomClick}
		>
			Custom
		</Button>
	</div>

	{#if !gui?.isDepositPreset(deposit.token_name)}
		<Input
			placeholder="Enter deposit amount"
			on:change={({ currentTarget }) => {
				if (currentTarget instanceof HTMLInputElement) {
					gui?.saveDeposit(deposit.token_name, currentTarget.value);
				}
			}}
		/>
	{/if}
</div>
