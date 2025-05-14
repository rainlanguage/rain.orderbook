<script lang="ts">
	import type { RemoveOrderTransactionState } from '$lib/models/RemoveOrderTransaction';
	import { type Writable } from 'svelte/store';
	export let state: Writable<RemoveOrderTransactionState>;
</script>

<div class="w-full max-w-md rounded-md p-2 shadow-sm">
	<p class="mb-2 break-words p-1">Status: {$state.status}</p>
	<p class="mb-2 break-words p-1">Message: {$state.message}</p>
	<p class="break-words p-1">
		Explorer Link: {$state.explorerLink}
	</p>
</div>
