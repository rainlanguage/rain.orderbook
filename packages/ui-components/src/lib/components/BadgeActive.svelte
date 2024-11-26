<script lang="ts">
	import { Badge } from 'flowbite-svelte';
	export let active: boolean;
</script>

{#if active}
	<Badge color="green" {...$$props}>Active</Badge>
{:else}
	<Badge color="yellow" {...$$props}>Inactive</Badge>
{/if}
