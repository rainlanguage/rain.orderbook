import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';

export type ValidStrategyDetail = {
	details: NameAndDescriptionCfg;
	name: string;
	dotrain: string;
};

export type InvalidStrategyDetail = {
	name: string;
	error: string;
};
