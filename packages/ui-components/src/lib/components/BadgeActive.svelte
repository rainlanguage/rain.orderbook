<script lang="ts">
    import {Badge} from "flowbite-svelte"
    export let active = false
</script>

<Badge color={active ? 'green' : 'yellow'} large>
	{active ? 'Active' : 'Inactive'}
</Badge>