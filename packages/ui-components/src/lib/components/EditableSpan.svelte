<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { PenSolid } from 'flowbite-svelte-icons';

	export let displayValue: string;

	let textContent: string;
	let editableSpan: HTMLSpanElement;

	let dispatch = createEventDispatcher();

	const inputBlurred = () => {
		displayValue = textContent;
		dispatch('blur', { value: textContent });
	};

	const handleKeyDown = (event: KeyboardEvent) => {
		if (event.key === 'Enter') {
			editableSpan.blur();
		}
	};
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
	data-testid="editableSpanWrapper"
	class="flex items-center gap-x-1 border-b-2 border-dotted text-sm text-gray-400 dark:text-gray-400"
>
	<PenSolid class="h-3 w-3 cursor-pointer" />
	<span>Block:</span>
	<div class="relative">
		<span
			data-testid="editableSpan"
			bind:this={editableSpan}
			bind:textContent
			contenteditable="true"
			on:keydown={handleKeyDown}
			on:blur={inputBlurred}>{displayValue}</span
		>
	</div>
</div>
