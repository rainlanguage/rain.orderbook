<script lang="ts">
	import { InputAddon, Button, Alert } from 'flowbite-svelte';
	import { InfoCircleSolid } from 'flowbite-svelte-icons';
	import { parseUnits } from 'viem';

	export let symbol: string | undefined = undefined;
	export let decimals: number = 0;
	export let maxValue: bigint | undefined = undefined;
	let inputValue: string = '';
	export let value: bigint = 0n;

	function handleInput(event: Event) {
		const input = event.target as HTMLInputElement;
		inputValue = input.value;

		if (inputValue === '') {
			value = 0n;
		} else {
			try {
				value = parseUnits(inputValue, decimals);
				// eslint-disable-next-line @typescript-eslint/no-unused-vars
			} catch (_e) {
				value = 0n;
			}
		}
	}

	function fillMaxValue() {
		if (!maxValue) return;

		value = maxValue;
		inputValue = maxValue.toString().padStart(decimals + 1, '0');
		inputValue = inputValue.slice(0, -decimals) + '.' + inputValue.slice(-decimals);
		inputValue = inputValue.replace(/\.?0+$/, '');
	}
</script>

<div class="w-full">
	<div class="flex w-full">
		<div class="relative flex w-full">
			<input
				type="text"
				class={`focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400 ${symbol && '!rounded-none !rounded-l-lg'}`}
				bind:value={inputValue}
				on:input={handleInput}
			/>

			{#if maxValue}
				<div class="absolute right-4 flex h-10 flex-col justify-center">
					<Button color="blue" class="px-2 py-1" size="xs" pill on:click={fillMaxValue}>MAX</Button>
				</div>
			{/if}
		</div>

		{#if symbol}
			<InputAddon>
				{symbol}
			</InputAddon>
		{/if}
	</div>
	{#if decimals === 0}
		<Alert color="yellow" border class="mt-2">
			<InfoCircleSolid slot="icon" class="h-6 w-6" />
			This token does not specify a number of decimals. <br />You are inputting the raw integer
			amount with 0 decimal places.
		</Alert>
	{/if}
</div>
