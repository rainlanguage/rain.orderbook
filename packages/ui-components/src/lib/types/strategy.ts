import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook';

export type ValidStrategyDetail = {
	details: NameAndDescriptionCfg;
	name: string;
	dotrain: string;
};

export type InvalidStrategyDetail = {
	name: string;
	error: string;
};
