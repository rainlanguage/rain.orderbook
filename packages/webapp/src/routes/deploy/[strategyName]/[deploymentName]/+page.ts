import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	try {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/strategies/dev/registry'
		);
		const files = await response.text();
        const { strategyName, deploymentName } = params;

		const fileList = files
			.split('\n')
			.filter(Boolean)
			.map(line => {
				const [name, url] = line.split(' ');
				return { name, url };
			});



		const strategy = fileList.find(file => file.name === strategyName);

		if (strategy) {
		const dotrainResponse = await fetch(strategy.url);
		const dotrain = await dotrainResponse.text();

		return {
			dotrain,
			strategyName,
			deploymentName
		};
	} else {
		return {
				dotrain: null,
                strategyName: null,
                deploymentName: null
			};
		}
	} catch {
		return {
			dotrain: null,
			strategyName: null,
			deploymentName: null
		};
	}
};
