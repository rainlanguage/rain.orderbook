import type {
	DotrainOrderGui,
	SelectTokens,
	GuiFieldDefinition,
	GuiDeposit,
	TokenInfos,
	Vault
} from '@rainlanguage/orderbook/js_api';

export type StepType = 'tokens' | 'fields' | 'deposits' | 'tokenInput' | 'tokenOutput';

export interface BaseWizardStep {
	type: StepType;
}

export interface SelectTokenStep extends BaseWizardStep {
	type: 'tokens';
	token: string;
	gui: DotrainOrderGui;
	selectTokens: SelectTokens;
}

export interface FieldStep extends BaseWizardStep {
	type: 'fields';
	fieldDefinition: GuiFieldDefinition;
	gui: DotrainOrderGui;
}

export interface DepositStep extends BaseWizardStep {
	type: 'deposits';
	deposit: GuiDeposit;
	gui: DotrainOrderGui;
	tokenInfos: TokenInfos;
}

export interface TokenInputStep extends BaseWizardStep {
	type: 'tokenInput';
	input: Vault;
	gui: DotrainOrderGui;
	tokenInfos: TokenInfos;
	i: number;
	inputVaultIds: string[];
}

export interface TokenOutputStep extends BaseWizardStep {
	type: 'tokenOutput';
	output: Vault;
	gui: DotrainOrderGui;
	tokenInfos: TokenInfos;
	i: number;
	outputVaultIds: string[];
}

export type WizardStep = TokenStep | FieldStep | DepositStep | TokenInputStep | TokenOutputStep;
