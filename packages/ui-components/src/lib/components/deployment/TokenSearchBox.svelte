<script lang="ts">
	import { createCombobox, melt, type ComboboxOptionProps } from '@melt-ui/svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import { fly } from 'svelte/transition';
	import type { ExtendedTokenInfo } from '../../types/tokens';
	import { createEventDispatcher } from 'svelte';

	export let tokenList: ExtendedTokenInfo[];

	const dispatch = createEventDispatcher<{
		select: ExtendedTokenInfo;
		input: string;
	}>();

	const toOption = (token: ExtendedTokenInfo): ComboboxOptionProps<ExtendedTokenInfo> => ({
		value: token,
		label: token.address || token.name || token.symbol,
		disabled: false
	});

	const {
		elements: { menu, input, option },
		states: { open, inputValue, touchedInput, selected },
		helpers: { isSelected }
	} = createCombobox<ExtendedTokenInfo>({
		forceVisible: true
	});

	$: if (!$open) {
		$inputValue = $selected?.label ?? '';
	}

	$: if ($selected) {
		dispatch('select', $selected.value);
	}
	$: if ($inputValue) {
		dispatch('input', $inputValue);
	}

	$: filteredTokens = $touchedInput
		? tokenList.filter((token) => {
				const normalizedInput = $inputValue.toLowerCase();
				return (
					token.name?.toLowerCase().includes(normalizedInput) ||
					token.symbol?.toLowerCase().includes(normalizedInput) ||
					token.address.toLowerCase().includes(normalizedInput)
				);
			})
		: tokenList;

	$: hasResults = filteredTokens.length > 0;
	$: showDropdown = $open && hasResults;
</script>

<input
	use:melt={$input}
	class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-3 text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 sm:text-base rtl:text-right dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
	placeholder="Search by name, symbol or address"
/>

{#if showDropdown}
	<ul
		class="z-10 flex max-h-[300px] flex-col divide-y divide-gray-100 overflow-hidden rounded-lg bg-white shadow-sm dark:bg-gray-700"
		use:melt={$menu}
		transition:fly={{ duration: 150, y: -5 }}
	>
		<div
			class="flex max-h-full flex-col gap-0 overflow-y-auto px-2 py-2 text-sm text-gray-700 dark:text-gray-200"
		>
			{#each filteredTokens as token, index (index)}
				<li
					use:melt={$option(toOption(token))}
					class="relative cursor-pointer scroll-my-2 rounded-md py-2 pl-4 pr-4 hover:bg-gray-100 data-[highlighted]:bg-gray-200 data-[highlighted]:text-gray-900 data-[disabled]:opacity-50 dark:hover:bg-gray-600 dark:hover:text-white"
				>
					{#if $isSelected(token)}
						<div class="check absolute left-2 top-1/2 z-10">
							<CheckCircleSolid class="size-4" />
						</div>
					{/if}
					<div class="pl-4">
						<span class="font-medium">{token.name || token.symbol}</span>
						<span class="block text-sm opacity-75">{token.address}</span>
					</div>
				</li>
			{/each}
		</div>
	</ul>
{/if}
