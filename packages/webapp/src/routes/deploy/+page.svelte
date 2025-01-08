<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from 'flowbite-svelte';
</script>

<div class="flex h-screen flex-col gap-4">
	<div class="mb-4">
		<Button on:click={() => goto('/deploy/test-strategy')}>Load Strategy</Button>
		<Button on:click={() => goto('/deploy/test-strategy-token-select')}
			>Load Token Select Strategy</Button
		>
	</div>
</div>
