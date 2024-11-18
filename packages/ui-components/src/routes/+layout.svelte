<script lang="ts">
	import '../lib/app.css';
</script>

<div
	class="flex h-[calc(100vh-2.5rem)] w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400"
>
	<main class="ml-64 h-full w-full grow overflow-x-auto p-8">
		<slot />
	</main>
</div>
