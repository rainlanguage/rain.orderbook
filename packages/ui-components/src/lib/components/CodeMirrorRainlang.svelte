<script lang="ts">
	import type { SgOrder } from '@rainlanguage/orderbook/js_api';
	import { extendOrder } from '@rainlanguage/orderbook/js_api';
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';

	export let order: SgOrder | undefined = undefined;
	export let rainlangText: string | undefined = undefined;
	export let codeMirrorTheme;
	export let codeMirrorDisabled = true;
	export let codeMirrorStyles = {};

	$: extendedOrder = order ? extendOrder(order) : undefined;
</script>

{#if extendedOrder?.rainlang}
	<CodeMirror
		value={rainlangText || extendedOrder.rainlang}
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
{:else if !extendedOrder?.rainlang && !rainlangText}
	<div
		class="w-full tracking-tight text-gray-900 dark:text-white"
		data-testid="rainlang-not-included"
	>
		Rain source not included in order meta
	</div>
{/if}
