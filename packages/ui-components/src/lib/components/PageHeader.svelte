<script lang="ts">
	import { Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
	import { generateBreadcrumbs } from '../utils/breadcrumbs';

	export let title: string;
	export let pathname: string;

	$: breadcrumbs = generateBreadcrumbs(pathname);
</script>

<div class="mb-8 flex w-full items-center">
	<div class="flex-1">
		<Breadcrumb
			olClass="inline-flex items-center rtl:space-x-reverse"
			aria-label="Default breadcrumb example"
		>
			<BreadcrumbItem href="/" home></BreadcrumbItem>
			{#each breadcrumbs as crumb}
				<BreadcrumbItem
					href={crumb.href}
					linkClass="mr-0 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white capitalize"
					>{crumb.label}</BreadcrumbItem
				>
			{/each}
			<BreadcrumbItem spanClass="text-sm font-medium text-gray-700 dark:text-gray-300 capitalize"
				><span data-testid="breadcrumb-page-title">{title}</span></BreadcrumbItem
			>
		</Breadcrumb>
	</div>
	<div class="flex-1">
		<div class="flex justify-end space-x-2">
			<slot name="actions" />
		</div>
	</div>
</div>
