<script lang="ts">
	export let title: string;
	export let description: string = '';
</script>

<div class="max-w-2xl text-left">
	<h1 class="break-words text-2xl font-bold text-gray-900 lg:text-3xl dark:text-white">{title}</h1>
	<p class="lg:text-2xltext-gray-600 break-words text-xl dark:text-gray-400">
		{description}
	</p>
</div>
