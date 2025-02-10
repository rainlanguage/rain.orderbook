<script lang="ts">
	export let strategyName: string;
	export let key: string;
	export let name: string;
	export let description: string;
</script>

<a href={`/deploy/${strategyName}/${key}`}>
	<button
		class="block max-w-sm cursor-pointer rounded-lg border border-gray-200 bg-white p-4 text-left shadow hover:bg-gray-100 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
	>
		<h1 class="text-2xl font-semibold text-gray-900 dark:text-white">{name}</h1>
		<p class="text-gray-600 dark:text-gray-400">{description}</p>
	</button>
</a>
