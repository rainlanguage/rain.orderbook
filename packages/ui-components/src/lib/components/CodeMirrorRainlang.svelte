<script lang="ts">
	import type { Order } from '@rainlanguage/orderbook/js_api';
	import { extendOrder } from '@rainlanguage/orderbook/js_api';

	export let order: Order | undefined = undefined;
	export let rainlangText: string = '';
	export let codeMirrorTheme;
	export let disabled = false;
	export let styles = {};

	$: value = order ? extendOrder(order).rainlang : rainlangText;
</script>

<slot name="codemirror" {value} {codeMirrorTheme} {disabled} {styles}></slot>
