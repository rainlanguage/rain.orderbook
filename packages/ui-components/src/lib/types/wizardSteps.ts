import type {
	DotrainOrderGui,
	SelectTokens,
	GuiFieldDefinition,
	TokenInfos,
	Vault,
	TokenDeposit,
	GuiPreset
} from '@rainlanguage/orderbook/js_api';

export interface SelectTokenStep {
	type: 'tokens';
	token: string;
	gui: DotrainOrderGui;
	selectTokens: SelectTokens;
}

export interface FieldStep {
	type: 'fields';
	fieldDefinition: GuiFieldDefinition;
	gui: DotrainOrderGui;
	fieldValue?: GuiPreset;
}

export interface DepositStep {
	type: 'deposits';
	deposit: TokenDeposit;
	gui: DotrainOrderGui;
	tokenInfos: TokenInfos;
}

export interface TokenInputStep {
	type: 'tokenInput';
	input: Vault;
	gui: DotrainOrderGui;
	tokenInfos: TokenInfos;
	i: number;
	inputVaultIds: string[];
}

export interface TokenOutputStep {
	type: 'tokenOutput';
	output: Vault;
	gui: DotrainOrderGui;
	tokenInfos: TokenInfos;
	i: number;
	outputVaultIds: string[];
}

export type WizardStep =
	| SelectTokenStep
	| FieldStep
	| DepositStep
	| TokenInputStep
	| TokenOutputStep;
