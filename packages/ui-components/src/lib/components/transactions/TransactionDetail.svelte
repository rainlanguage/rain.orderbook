<script lang="ts">
	import type { RemoveOrderTransactionState } from '$lib/models/RemoveOrderTransaction';
	import { type Writable } from 'svelte/store';
	export let state: Writable<RemoveOrderTransactionState>;
</script>

<div class="rounded-md border border-gray-300 p-4 shadow-sm outline outline-1 outline-gray-200">
	<p class="mb-2 p-2 outline outline-1 outline-gray-100">Status: {$state.status}</p>
	<p class="mb-2 p-2 outline outline-1 outline-gray-100">Message: {$state.message}</p>
	<p class="p-2 outline outline-1 outline-gray-100">Explorer Link: {$state.explorerLink}</p>
</div>
