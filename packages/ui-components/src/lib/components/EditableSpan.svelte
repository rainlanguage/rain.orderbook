<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { PenSolid } from 'flowbite-svelte-icons';

	export let displayValue: string;

	let focussed: boolean = false;
	let textContent: string;
	let editableSpan: HTMLSpanElement;

	let dispatch = createEventDispatcher();

	const inputFocussed = () => {
		focussed = true;
		editableSpan.focus();
		dispatch('focus');
	};

	const inputBlurred = () => {
		focussed = false;
		displayValue = textContent;
		dispatch('blur', { value: textContent });
	};

	const handleKeyDown = (event: KeyboardEvent) => {
		if (event.key === 'Enter') {
			editableSpan.blur();
		}
	};
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
	data-testid="editableSpanWrapper"
	on:click={inputFocussed}
	class="flex items-center gap-x-1 border-b-2 border-dotted text-sm text-gray-400 dark:text-gray-400"
>
	<PenSolid class="h-3 w-3 cursor-pointer" />
	<span>Block:</span>
	<div class="relative">
		<span
			data-testid="editableSpan"
			class="absolute"
			class:opacity-0={!focussed}
			bind:this={editableSpan}
			bind:textContent
			contenteditable="true"
			on:keydown={handleKeyDown}
			on:blur={inputBlurred}>{displayValue}</span
		>
		<span
			data-testid="displayElement"
			class="pointer-events-none"
			class:opacity-0={focussed}
			on:click={inputFocussed}>{displayValue}</span
		>
	</div>
</div>
