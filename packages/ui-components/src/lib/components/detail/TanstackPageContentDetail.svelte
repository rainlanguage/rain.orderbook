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
	</div>
	<div class="grid gap-4 lg:grid-cols-3">
		<div class="col-span-1 flex flex-col gap-y-6">
			<slot name="card" {data} />
		</div>
		<div class="col-span-2 min-h-[500px]">
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
