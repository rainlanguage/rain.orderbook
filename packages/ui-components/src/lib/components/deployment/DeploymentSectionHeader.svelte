<script lang="ts">
	export let title: string;
	export let description: string = '';
</script>

<div class="max-w-2xl text-left">
	<h1 class="mb-4 text-4xl font-bold text-gray-900 dark:text-white">{title}</h1>
	<p class="text-xl text-gray-600 dark:text-gray-400">
		{description}
	</p>
</div>
