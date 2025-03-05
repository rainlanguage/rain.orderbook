<script lang="ts">
	export let title: string;
	export let description: string = '';
</script>

<div class="flex max-w-xl flex-grow flex-col gap-y-4 text-left">
	<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-2xl dark:text-white">
		{title}
	</h1>
	<p class="break-words text-sm font-light text-gray-600 lg:text-base dark:text-gray-400">
		{description}
	</p>
</div>
