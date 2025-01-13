<script lang="ts">
	import { Homepage } from '@rainlanguage/ui-components';
	import { colorTheme } from '$lib/darkMode';

	$: console.log($colorTheme);
</script>

<Homepage {colorTheme} />
