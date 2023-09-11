<script lang="ts">
	export let label: string;
</script>

<div class="flex flex-col">
	{#if label}
		<p class="label">{label}</p>
	{/if}
	<p>
		<slot />
	</p>
</div>

<style lang="postcss">
	.label {
		@apply block text-sm font-semibold text-gray-400;
	}
</style>
