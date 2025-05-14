<script lang="ts">
	import type { RemoveOrderTransactionState } from '$lib/models/RemoveOrderTransaction';
	import { type Writable } from 'svelte/store';
	export let state: Writable<RemoveOrderTransactionState>;
</script>

<div class="w-full max-w-md rounded-md p-2 shadow-sm">
	<p class="mb-2 break-words p-1">Message: {$state.message}</p>
	<p class="break-words p-1">
		<a
			href={$state.explorerLink}
			target="_blank"
			rel="noopener noreferrer"
			class="text-blue-500 hover:underline">View transaction on explorer</a
		>
	</p>
</div>
