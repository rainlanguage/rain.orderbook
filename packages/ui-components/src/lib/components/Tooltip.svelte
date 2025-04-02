<script lang="ts">
	import { Tooltip } from 'flowbite-svelte';
	export let triggeredBy: string;
	export let customClass: string = '';
</script>

<Tooltip {triggeredBy} class={`z-[99] shadow-none ${customClass}`}>
	<slot />
</Tooltip>
