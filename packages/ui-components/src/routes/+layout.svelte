<script lang="ts">
	import { queryClient } from '$lib/queries/queryClient';
	import { QueryClientProvider } from '@tanstack/svelte-query';
	import '../app.css';
</script>

<QueryClientProvider client={queryClient}>
	<main class="ml-64 h-full w-full grow overflow-x-auto p-8">
		<slot />
	</main>
</QueryClientProvider>
