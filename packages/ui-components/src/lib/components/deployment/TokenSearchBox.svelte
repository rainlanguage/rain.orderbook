<script lang="ts">
	import { createCombobox, melt, type ComboboxOptionProps } from '@melt-ui/svelte';
	import { CheckCircleSolid, ChevronDownSolid, ChevronUpSolid } from 'flowbite-svelte-icons';
	import { fly } from 'svelte/transition';
	import type { ExtendedTokenInfo } from '../../types/tokens';

	export let tokenList: ExtendedTokenInfo[];

	$: console.log('tokenList:', tokenList);

	const toOption = (token: ExtendedTokenInfo): ComboboxOptionProps<ExtendedTokenInfo> => ({
		value: token,
		label: token.name || token.symbol || token.address,
		disabled: false
	});

	const {
		elements: { menu, input, option, label },
		states: { open, inputValue, touchedInput, selected },
		helpers: { isSelected }
	} = createCombobox<ExtendedTokenInfo>({
		forceVisible: true
	});

	$: if (!$open) {
		$inputValue = $selected?.label ?? '';
	}

	$: {
		console.log('tokenList:', tokenList);
		console.log('touchedInput:', $touchedInput);
		console.log('inputValue:', $inputValue);
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

	$: console.log('filteredTokens:', filteredTokens);
</script>

<div class="flex flex-col gap-1">
	<!-- svelte-ignore a11y-label-has-associated-control - $label contains the 'for' attribute -->
	<label use:melt={$label}>
		<span class="text-sm font-medium">Select a token:</span>
	</label>

	<div class="relative">
		<input
			use:melt={$input}
			class="flex h-10 items-center justify-between rounded-lg bg-white
          px-3 pr-12 text-black"
			placeholder="Search by name, symbol or address"
		/>
		<div class="absolute right-2 top-1/2 z-10 -translate-y-1/2">
			{#if $open}
				<ChevronUpSolid class="size-4" />
			{:else}
				<ChevronDownSolid class="size-4" />
			{/if}
		</div>
	</div>
</div>
{#if $open}
	<ul
		class="z-10 flex max-h-[300px] flex-col overflow-hidden rounded-lg"
		use:melt={$menu}
		transition:fly={{ duration: 150, y: -5 }}
	>
		<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
		<div
			class="flex max-h-full flex-col gap-0 overflow-y-auto bg-white px-2 py-2 text-black"
			tabindex="0"
		>
			{#each filteredTokens as token, index (index)}
				<li
					use:melt={$option(toOption(token))}
					class="relative cursor-pointer scroll-my-2 rounded-md py-2 pl-4 pr-4
        hover:bg-gray-100
        data-[highlighted]:bg-gray-200 data-[highlighted]:text-gray-900
          data-[disabled]:opacity-50"
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
			{:else}
				<li class="relative cursor-pointer rounded-md py-1 pl-8 pr-4">No results found</li>
			{/each}
		</div>
	</ul>
{/if}
