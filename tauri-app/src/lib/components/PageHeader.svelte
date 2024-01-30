<script lang="ts">
  import { Breadcrumb, BreadcrumbItem } from "flowbite-svelte";

  export let title: string;
</script>

<div class="flex w-full items-center mb-8">
  <div class="flex-1">
    <Breadcrumb aria-label="Default breadcrumb example">
      <BreadcrumbItem href="/" home></BreadcrumbItem>
      <slot name="breadcrumbs"></slot>
      <BreadcrumbItem spanClass="ms-1 text-sm font-medium text-gray-700 md:ms-2 dark:text-gray-300">{title}</BreadcrumbItem>
    </Breadcrumb>
  </div>
  <h1 class="flex-0 text-4xl font-bold text-gray-900 dark:text-white">{title}</h1>
  <div class="flex-1">
    <div class="flex justify-end space-x-2">
      <slot name="actions"></slot>
    </div>
  </div>
</div>