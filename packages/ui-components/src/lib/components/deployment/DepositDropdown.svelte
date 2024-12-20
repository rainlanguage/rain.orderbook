<script lang="ts">
	import {
		DotrainOrderGui,
		type GuiDeposit,
		type TokenInfos
	} from '@rainlanguage/orderbook/js_api';
	import DropdownRadio from '../dropdown/DropdownRadio.svelte';
	import { Label, Input } from 'flowbite-svelte';

	export let deposit: GuiDeposit;
	export let gui: DotrainOrderGui;
	export let tokenInfos: TokenInfos;
</script>

<div class="mb-4 flex flex-col gap-2">
	<Label class="whitespace-nowrap text-xl">{tokenInfos.get(deposit.token.address)?.name}</Label>

	<DropdownRadio
		options={{
			...Object.fromEntries(
				deposit.presets.map((preset) => [
					preset,
					{
						label: preset
					}
				])
			),
			...{ custom: { label: 'Custom value' } }
		}}
		on:change={({ detail }) => {
			gui?.saveDeposit(deposit.token_name, detail.value === 'custom' ? '' : detail.value || '');
			gui = gui;
		}}
	>
		<svelte:fragment slot="content" let:selectedOption let:selectedRef>
			{#if selectedRef === undefined}
				<span>Choose deposit amount</span>
			{:else if selectedOption?.label}
				<span>{selectedOption.label}</span>
			{:else}
				<span>{selectedRef}</span>
			{/if}
		</svelte:fragment>

		<svelte:fragment slot="option" let:option let:ref>
			<div class="w-full overflow-hidden overflow-ellipsis">
				<div class="text-md break-word">{option.label ? option.label : ref}</div>
			</div>
		</svelte:fragment>
	</DropdownRadio>

	{#if gui?.isDepositPreset(deposit.token_name) === false}
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
