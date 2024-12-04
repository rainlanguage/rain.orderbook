<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';
	import type { Order } from '@rainlanguage/orderbook/js_api';
	import { extendOrder } from '@rainlanguage/orderbook/js_api';
	import pkg from '@rainlanguage/dotrain';
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const { MetaStore, RainLanguageServices, TextDocumentItem, CompletionItemKind } = pkg;
	export let order: Order | undefined = undefined;
	export let rainlangText: string = '';
	export let disabled = false;
	export let styles = {};
	export let codeMirrorTheme;

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
