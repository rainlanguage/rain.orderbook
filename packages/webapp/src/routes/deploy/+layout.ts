import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
	const owner = 'rainlanguage';
	const repo = 'rain.strategies';
	const path = 'strategies/dev';

	try {
		const response = await fetch(`https://api.github.com/repos/${owner}/${repo}/contents/${path}`);
		const data = await response.json();

		return {
			files: data
		};
	} catch {
		return {
			files: []
		};
	}
};
