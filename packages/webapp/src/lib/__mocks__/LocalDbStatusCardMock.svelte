<script lang="ts">
	export let status: string = 'active';
	export let error: string | undefined = undefined;
</script>

<div data-testid="local-db-status-card">
	{#if error && status === 'failure'}
		<button data-testid="local-db-error-copy">Copy error details</button>
	{/if}
</div>
