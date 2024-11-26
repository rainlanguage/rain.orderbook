import { darkCodeMirrorTheme, lightCodeMirrorTheme } from '$lib/utils/codeMirrorThemes';
import { darkChartTheme, lightChartTheme } from '$lib/utils/lightweightChartsThemes';
import { derived, writable } from 'svelte/store';

function readColorTheme(): 'dark' | 'light' {
	const saved = localStorage.getItem('color-theme');
	if (saved) {
		return saved as 'dark' | 'light';
	} else if (document.body.classList.contains('dark')) {
		return 'dark';
	} else {
		return 'light';
	}
}

export const colorTheme = writable(readColorTheme());

colorTheme.subscribe((val: string) => localStorage.setItem('color-theme', val));

export const codeMirrorTheme = derived(colorTheme, ($colorTheme) =>
	$colorTheme === 'dark' ? darkCodeMirrorTheme : lightCodeMirrorTheme
);

export const lightweightChartsTheme = derived(colorTheme, ($colorTheme) =>
	$colorTheme === 'dark' ? darkChartTheme : lightChartTheme
);
