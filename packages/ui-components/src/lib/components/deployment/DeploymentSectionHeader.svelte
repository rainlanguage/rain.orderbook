<script lang="ts">
	export let title: string;
	export let description: string = '';
</script>

<div class="mt-16 max-w-2xl text-center">
	<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">{title}</h1>
	<p class="mb-12 text-xl text-gray-600 dark:text-gray-400">
		{description}
	</p>
</div>
