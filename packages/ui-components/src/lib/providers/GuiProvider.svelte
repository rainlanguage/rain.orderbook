<script lang="ts">
	import { setContext } from 'svelte';
	import { RaindexOrderBuilder } from '@rainlanguage/orderbook';
	import { GUI_CONTEXT_KEY } from '../hooks/useGui';

	export let gui: RaindexOrderBuilder;

	setContext(GUI_CONTEXT_KEY, gui);
</script>

<slot />
