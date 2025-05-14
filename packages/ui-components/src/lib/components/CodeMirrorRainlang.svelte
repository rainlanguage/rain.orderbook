<script lang="ts">
	import type { SgOrder } from '@rainlanguage/orderbook';
	import { extendOrder } from '@rainlanguage/orderbook';
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';

	export let order: SgOrder | undefined = undefined;
	export let rainlangText: string | undefined = undefined;
	export let codeMirrorTheme;
	export let codeMirrorDisabled = true;
	export let codeMirrorStyles = {};

	let result;
	let extendedOrder;

	$: result = order ? extendOrder(order) : undefined;
	$: extendedOrder = result && !result.error ? result.value : undefined;
	$: extendOrderError = result && result.error ? result.error : undefined;
</script>

{#if rainlangText || extendedOrder?.rainlang}
	<CodeMirror
		value={rainlangText || extendedOrder?.rainlang}
		extensions={[RainlangLR]}
		theme={codeMirrorTheme}
		readonly={codeMirrorDisabled}
		useTab={true}
		tabSize={2}
		styles={{
			'&': {
				width: '100%'
			},
			...codeMirrorStyles
		}}
	/>
{:else if extendOrderError}
	<div class="w-full tracking-tight text-red-600 dark:text-red-400" data-testid="rainlang-error">
		{extendOrderError.readableMsg}
	</div>
{:else if !extendedOrder?.rainlang && !rainlangText}
	<div
		class="w-full tracking-tight text-gray-900 dark:text-white"
		data-testid="rainlang-not-included"
	>
		Rain source not included in order meta
	</div>
{/if}
