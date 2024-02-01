<script lang="ts">
  import { Breadcrumb, BreadcrumbItem } from "flowbite-svelte";
  import { page } from '$app/stores';
  import { generateBreadcrumbs } from "$lib/utils/breadcrumbs";

  export let title: string;

  $: breadcrumbs = generateBreadcrumbs($page.url.pathname);
</script>

<div class="flex w-full items-center mb-8">
  <div class="flex-1">
    <Breadcrumb aria-label="Default breadcrumb example">
      <BreadcrumbItem href="/" home></BreadcrumbItem>
      {#each breadcrumbs as crumb}
        <BreadcrumbItem href={crumb.href} linkClass="ms-1 text-sm font-medium text-gray-700 hover:text-gray-900 md:ms-2 dark:text-gray-400 dark:hover:text-white capitalize">{crumb.label}</BreadcrumbItem>
      {/each}
      <BreadcrumbItem spanClass="ms-1 text-sm font-medium text-gray-700 md:ms-2 dark:text-gray-300 capitalize">{title}</BreadcrumbItem>
    </Breadcrumb>
  </div>
  <h1 class="flex-0 text-4xl font-bold text-gray-900 dark:text-white">{title}</h1>
  <div class="flex-1">
    <div class="flex justify-end space-x-2">
      <slot name="actions"></slot>
    </div>
  </div>
</div>