<script lang="ts" generics="T">
	import type { CreateQueryResult } from '@tanstack/svelte-query';
	import { Spinner } from 'flowbite-svelte';

	// eslint-disable-next-line no-undef
	export let query: CreateQueryResult<T>;
	export let emptyMessage = 'Not found';

	// We need to explicitly define the data type as non-nullable here because
	// doing it in the component body ({#if $query.data}) doesn't make the slot
	// prop non-nullable when used in the parent component.

	// eslint-disable-next-line no-undef
	let data: NonNullable<T>;
	$: if ($query.data) {
		data = $query.data;
	}
</script>

{#if data}
	<div class="mb-6 flex items-end justify-between">
		<slot name="top" {data} />
		<slot name="action-buttons" {data} />
	</div>
	<div class="tanstack-detail-grid">
		<div class="flex flex-col gap-y-6 lg:col-span-1">
			<slot name="card" {data} />
		</div>
		<div class="h-[500px] lg:col-span-2">
			<slot name="chart" {data} />
		</div>
	</div>
	<div class="w-full">
		<slot name="below" {data} />
	</div>
{:else if $query.isFetching || $query.isLoading}
	<div class="flex h-16 w-full items-center justify-center">
		<Spinner class="h-8 w-8" color="white" data-testid="loadingSpinner" />
	</div>
{:else}
	<div data-testid="emptyMessage" class="text-center text-gray-900 dark:text-white">
		{emptyMessage}
	</div>
{/if}

<style>
	.tanstack-detail-grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		width: 100%;
	}

	@media (min-width: 1024px) {
		.tanstack-detail-grid {
			flex-direction: row;
		}

		.tanstack-detail-grid > :first-child {
			flex: 1;
		}

		.tanstack-detail-grid > :last-child {
			flex: 2;
		}
	}
</style>
