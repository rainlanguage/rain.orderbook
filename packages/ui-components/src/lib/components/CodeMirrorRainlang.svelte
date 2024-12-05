<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';
	import type { Order } from '@rainlanguage/orderbook/js_api';
	import { extendOrder } from '@rainlanguage/orderbook/js_api';

	export let order: Order | undefined = undefined;
	export let rainlangText: string = '';
	export let disabled = false;
	export let styles = {};
	export let codeMirrorTheme;

	$: console.log(order);

	$: value = order ? extendOrder(order).rainlang : rainlangText;
</script>

<CodeMirror
	bind:value
	extensions={[RainlangLR]}
	theme={codeMirrorTheme}
	readonly={disabled}
	useTab={true}
	tabSize={2}
	styles={{
		'&': {
			width: '100%'
		},
		...styles
	}}
/>
