<script lang="ts" generics="DataItem, InputData = DataItem[]">
	import { invalidateTanstackQueries } from '$lib/queries/queryClient';
	import Refresh from './icon/Refresh.svelte';
	import type { CreateInfiniteQueryResult, InfiniteData } from '@tanstack/svelte-query';
	import { Button, Table, TableBody, TableBodyRow, TableHead } from 'flowbite-svelte';
	import { afterUpdate, createEventDispatcher, onDestroy, onMount } from 'svelte';
	import { useQueryClient } from '@tanstack/svelte-query';

	const queryClient = useQueryClient();

	const dispatch = createEventDispatcher();

	export let queryKey: string;
	export let query: CreateInfiniteQueryResult<InfiniteData<InputData, unknown>, Error>;
	export let emptyMessage: string = 'None found';
	export let rowHoverable = true;
	// Selector to extract DataItem[] from each page of type InputData
	export let dataSelector: (pageData: InputData) => DataItem[] = (pageData) =>
		Array.isArray(pageData) ? (pageData as unknown as DataItem[]) : [];
	// Virtualization controls
	export let enableVirtualization = true;
	export let estimatedRowHeight = 56;
	export let virtualizationOverscan = 8;

// Transform the query data by applying dataSelector to each page only when the page
// reference or selector changes. This keeps the array reference stable so large tables
// don't re-render on every fetch status update.
let transformedPages: DataItem[][] = [];
let lastPagesRef: InputData[] | undefined;
let lastSelector = dataSelector;
$: {
	const currentData = $query.data;
	const currentPages = currentData?.pages;
	const selectorChanged = lastSelector !== dataSelector;
	if (!currentPages) {
		transformedPages = [];
		lastPagesRef = undefined;
		lastSelector = dataSelector;
	} else if (currentPages !== lastPagesRef || selectorChanged) {
		transformedPages = currentPages.map((page) => dataSelector(page));
		lastPagesRef = currentPages;
		lastSelector = dataSelector;
	}
}

// Helper derived values kept out of the main script block for readability
$: firstPageLength = transformedPages?.[0]?.length ?? 0;
$: hasData = transformedPages.length > 0;
$: flattenedRows = transformedPages.flat();
$: totalRows = flattenedRows.length;

// Virtualization state tied to the window scroll so the table keeps its natural height.
const hasWindow = typeof window !== 'undefined';
const supportsResizeObserver = hasWindow && 'ResizeObserver' in window;
let tableContainerElement: HTMLDivElement | null = null;
let tableContainerObserver: ResizeObserver | null = null;
let tableOffsetTop = 0;
let windowScrollY = hasWindow ? window.scrollY : 0;
let windowHeight = hasWindow ? window.innerHeight : 0;
let measuredRowHeight: number | null = null;
$: rowHeight = Math.max(1, measuredRowHeight ?? estimatedRowHeight);
$: if (!enableVirtualization && measuredRowHeight !== null) {
	measuredRowHeight = null;
}

function updateTableOffset() {
	if (!enableVirtualization || !hasWindow || !tableContainerElement) {
		tableOffsetTop = 0;
		return;
	}
	const rect = tableContainerElement.getBoundingClientRect();
	tableOffsetTop = rect.top + window.scrollY;
}

function observeTableContainer(element: HTMLDivElement | null) {
	if (!supportsResizeObserver) {
		tableContainerObserver?.disconnect();
		tableContainerObserver = null;
		return;
	}
	if (tableContainerObserver && element === tableContainerElement) {
		return;
	}
	tableContainerObserver?.disconnect();
	if (element) {
		tableContainerObserver = new ResizeObserver(() => {
			updateTableOffset();
		});
		tableContainerObserver.observe(element);
	} else {
		tableContainerObserver = null;
	}
}

onMount(() => {
	if (!hasWindow) {
		return;
	}
	const handleScroll = () => {
		windowScrollY = window.scrollY;
	};
	const handleResize = () => {
		windowHeight = window.innerHeight;
		updateTableOffset();
	};
	window.addEventListener('scroll', handleScroll, { passive: true });
	window.addEventListener('resize', handleResize);
	windowScrollY = window.scrollY;
	windowHeight = window.innerHeight;
	updateTableOffset();
	return () => {
		window.removeEventListener('scroll', handleScroll);
		window.removeEventListener('resize', handleResize);
	};
});

$: if (enableVirtualization && tableContainerElement) {
	updateTableOffset();
	observeTableContainer(tableContainerElement);
} else if (!enableVirtualization) {
	tableContainerObserver?.disconnect();
	tableContainerObserver = null;
}

onDestroy(() => {
	tableContainerObserver?.disconnect();
});

let startIndex = 0;
let endIndex = 0;
let visibleRows: DataItem[] = [];
let topPadding = 0;
let bottomPadding = 0;
let shouldMeasureRowHeight = false;
$: {
	const virtualizationEnabled = enableVirtualization && totalRows > 0;
	const viewportTop = virtualizationEnabled ? windowScrollY : 0;
	const relativeViewportTop = virtualizationEnabled ? viewportTop - tableOffsetTop : 0;
	const estimatedVisibleCount = virtualizationEnabled
		? Math.ceil(windowHeight / rowHeight) + virtualizationOverscan * 2
		: totalRows;
	startIndex = virtualizationEnabled
		? Math.max(0, Math.min(totalRows, Math.floor(relativeViewportTop / rowHeight) - virtualizationOverscan))
		: 0;
	endIndex = virtualizationEnabled
		? Math.min(totalRows, Math.max(startIndex, startIndex + estimatedVisibleCount))
		: totalRows;
	visibleRows = virtualizationEnabled ? flattenedRows.slice(startIndex, endIndex) : flattenedRows;
	topPadding = virtualizationEnabled ? startIndex * rowHeight : 0;
	bottomPadding = virtualizationEnabled
		? Math.max(0, totalRows * rowHeight - topPadding - visibleRows.length * rowHeight)
		: 0;
	shouldMeasureRowHeight = virtualizationEnabled && visibleRows.length > 0;
}

afterUpdate(() => {
	if (!shouldMeasureRowHeight || !tableContainerElement) {
		return;
	}
	const firstRow = tableContainerElement.querySelector('tbody tr[data-virtual-row="true"]');
	if (!firstRow) {
		return;
	}
	const height = firstRow.getBoundingClientRect().height;
	if (height > 0) {
		measuredRowHeight =
			measuredRowHeight === null
				? height
				: Math.abs(measuredRowHeight - height) > 0.5
					? height
					: measuredRowHeight;
	}
});
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
{#if firstPageLength === 0}
	<div data-testid="emptyMessage" class="text-center text-gray-900 dark:text-white">
		{emptyMessage}
	</div>
{:else if hasData}
	<div class="cursor-pointer" bind:this={tableContainerElement}>
		<Table divClass="min-w-full rounded-lg border overflow-hidden dark:border-none" hoverable={rowHoverable}>
			<TableHead data-testid="head">
				<slot name="head" />
			</TableHead>
			<TableBody>
				{#if enableVirtualization && topPadding > 0}
					<tr aria-hidden="true">
						<td colspan="1000" class="border-0 p-0" style={`height:${topPadding}px;`} />
					</tr>
				{/if}
				{#if enableVirtualization}
					{#each visibleRows as item, index (startIndex + index)}
						<TableBodyRow
							class="whitespace-nowrap"
							data-testid="bodyRow"
							data-virtual-row="true"
							on:click={() => {
								dispatch('clickRow', { item });
							}}
						>
							<slot name="bodyRow" {item} />
						</TableBodyRow>
					{/each}
				{:else}
					{#each transformedPages as page}
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
				{/if}
				{#if enableVirtualization && bottomPadding > 0}
					<tr aria-hidden="true">
						<td colspan="1000" class="border-0 p-0" style={`height:${bottomPadding}px;`} />
					</tr>
				{/if}
			</TableBody>
		</Table>
	</div>
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
