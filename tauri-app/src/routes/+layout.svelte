<script>
	import '../app.pcss';
	import Sidebar from '../lib/Sidebar.svelte';
</script>

<div class="flex min-h-screen w-full grow-0 justify-start bg-white p-2 dark:bg-gray-600">
	<Sidebar />
	<main class="h-full w-full grow p-4">
		<slot />
	</main>
</div>
