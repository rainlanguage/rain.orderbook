<script lang="ts">
	import { setContext } from 'svelte';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;

	const GUI_CONTEXT_KEY = 'gui-context';

	setContext(GUI_CONTEXT_KEY, gui);
</script>

<slot />
