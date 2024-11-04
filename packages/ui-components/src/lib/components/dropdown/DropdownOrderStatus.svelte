<script lang="ts">
  import DropdownCheckbox from './DropdownCheckbox.svelte';
  import { activeOrderStatus } from '$lib/stores/settings';

  const orderStatusOptions = {
    Active: 'active',
    Inactive: 'inactive',
  };

  function handleStatusChange(event: CustomEvent<Record<string, string>>) {
    let status: boolean | undefined = undefined;
    let items = Object.keys(event.detail);

    if (items.length === 0 || items.length === 2) {
      status = undefined;
    } else if (items.includes('Active')) {
      status = true;
    } else if (items.includes('Inactive')) {
      status = false;
    }

    activeOrderStatus.set(status);
  }

  $: value = (
    $activeOrderStatus === undefined
      ? {}
      : $activeOrderStatus
        ? { Active: 'active' }
        : { Inactive: 'inactive' }
  ) as Record<string, string>;
</script>

<DropdownCheckbox
  options={orderStatusOptions}
  on:change={handleStatusChange}
  label="Status"
  showAllLabel={false}
  onlyTitle={true}
  {value}
/>
