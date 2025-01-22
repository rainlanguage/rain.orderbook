import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	try {
		const response = await fetch(`https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/strategies/dev/registry.json`);
		const data = await response.json();

		return {
			strategies: data
		};
	} catch {
		return {
			strategies: []
		};
	}
};
