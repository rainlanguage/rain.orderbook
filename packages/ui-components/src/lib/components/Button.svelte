<script lang="ts">
	import { Button } from 'flowbite-svelte';


	export let customClass: string = '';
    export let defaultClass = "border-1 border-grey-300"
</script>

<Button {...$$restProps} class={defaultClass} on:click>
	<slot />
</Button>
