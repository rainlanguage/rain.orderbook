<script lang="ts">
	import { Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
	import { generateBreadcrumbs } from '../utils/breadcrumbs';

	export let title: string;
	export let pathname: string;

	$: breadcrumbs = generateBreadcrumbs(pathname);
</script>

<div class="mb-4 flex w-full items-center">
	<div class="flex-grow">
		<Breadcrumb
			olClass="inline-flex items-center rtl:space-x-reverse"
			aria-label="Default breadcrumb example"
		>
			<BreadcrumbItem href="/" home></BreadcrumbItem>
			{#each breadcrumbs as crumb}
				<BreadcrumbItem
					href={crumb.href}
					linkClass="mx-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white first-letter:uppercase"
					>{crumb.label}</BreadcrumbItem
				>
			{/each}
			<BreadcrumbItem
				spanClass="mx-2 text-sm font-medium text-gray-700 dark:text-gray-300 first-letter:uppercase"
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
