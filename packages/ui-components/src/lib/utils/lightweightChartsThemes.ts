import { ColorType } from 'lightweight-charts';

export const darkChartTheme = {
	layout: {
		textColor: 'white',
		background: { type: ColorType.Solid, color: 'transparent' }
	},
	grid: {
		vertLines: { color: '#484848' },
		horzLines: { color: '#484848' }
	}
};

export const lightChartTheme = {
	layout: {
		textColor: 'black',
		background: { type: ColorType.Solid, color: 'transparent' }
	},
	grid: {
		vertLines: { color: '#ECECEC' },
		horzLines: { color: '#ECECEC' }
	}
};
