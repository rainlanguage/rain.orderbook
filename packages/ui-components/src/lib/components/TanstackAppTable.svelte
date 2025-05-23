<script lang="ts" generics="T">
	import { invalidateTanstackQueries } from '$lib/queries/queryClient';
	import Refresh from './icon/Refresh.svelte';
	import type { CreateInfiniteQueryResult, InfiniteData } from '@tanstack/svelte-query';
	import { Button, Table, TableBody, TableBodyRow, TableHead } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import { useQueryClient } from '@tanstack/svelte-query';

	const queryClient = useQueryClient();

	const dispatch = createEventDispatcher();

	export let queryKey: string;
	// eslint-disable-next-line no-undef
	export let query: CreateInfiniteQueryResult<InfiniteData<T[], unknown>, Error>;
	export let emptyMessage: string = 'None found';
	export let rowHoverable = true;
</script>

<div data-testid="title" class="flex h-16 w-full items-center justify-end">
	<slot name="info" />
	<slot name="timeFilter" />
	<slot name="title" />
	<Refresh
		class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
		data-testid="refreshButton"
		spin={$query.isLoading || $query.isFetching}
		on:click={async () => {
			if (queryKey) {
				invalidateTanstackQueries(queryClient, [queryKey]);
			}
		}}
	/>
</div>
{#if $query.data?.pages[0].length === 0}
	<div data-testid="emptyMessage" class="text-center text-gray-900 dark:text-white">
		{emptyMessage}
	</div>
{:else if $query.data}
	<Table
		divClass="cursor-pointer rounded-lg overflow-auto dark:border-none border"
		hoverable={rowHoverable}
	>
		<TableHead data-testid="head">
			<slot name="head" />
		</TableHead>
		<TableBody>
			{#each $query.data?.pages as page}
				{#each page as item}
					<TableBodyRow
						class="whitespace-nowrap"
						data-testid="bodyRow"
						on:click={() => {
							dispatch('clickRow', { item });
						}}
					>
						<slot name="bodyRow" {item} />
					</TableBodyRow>
				{/each}
			{/each}
		</TableBody>
	</Table>
	<div class="mt-2 flex justify-center">
		<Button
			data-testid="loadMoreButton"
			size="xs"
			color="dark"
			on:click={() => $query.fetchNextPage()}
			disabled={!$query.hasNextPage || $query.isFetchingNextPage}
		>
			{#if $query.isFetchingNextPage}
				Loading more...
			{:else if $query.hasNextPage}
				Load More
			{:else}Nothing more to load{/if}
		</Button>
	</div>
{/if}
