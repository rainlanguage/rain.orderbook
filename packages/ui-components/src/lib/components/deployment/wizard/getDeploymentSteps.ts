import type { TokenOutputStep } from '$lib/types/wizardSteps';

import type { DepositStep, FieldStep, TokenInputStep } from '$lib/types/wizardSteps';

import type { SelectTokenStep } from '$lib/types/wizardSteps';

import type {
	DotrainOrderGui,
	GuiFieldDefinition,
	TokenInfos,
	Vault,
	GuiDeposit
} from '@rainlanguage/orderbook/js_api';
import type { WizardStep } from '../../../types/wizardSteps';

// TODO: Add a check to see if the user has selected to use custom token input/outputs
// If they have, then we need to add a step to the wizard to allow them to enter the vault ids
// If they have not, then we can skip this step

export const getDeploymentSteps = (
	selectTokens: Map<string, string>,
	isLimitStrat: boolean,
	allFieldDefinitions: GuiFieldDefinition[],
	gui: DotrainOrderGui,
	allDeposits: GuiDeposit[],
	allTokenInputs: Vault[],
	allTokenOutputs: Vault[],
	inputVaultIds: string[],
	outputVaultIds: string[],
	tokenInfos: TokenInfos
) => {
	const deploymentSteps: WizardStep[] = [
		...(selectTokens.size > 0 && isLimitStrat
			? Array.from(selectTokens.entries()).map(
					([token]): SelectTokenStep => ({
						type: 'tokens',
						token,
						gui,
						selectTokens
					})
				)
			: []),

		...allFieldDefinitions.map(
			(fieldDefinition): FieldStep => ({
				type: 'fields',
				fieldDefinition,
				gui
			})
		),

		...allDeposits.map(
			(deposit): DepositStep => ({
				type: 'deposits',
				deposit,
				gui,
				tokenInfos
			})
		),

		...allTokenInputs.map(
			(input, i): TokenInputStep => ({
				type: 'tokenInput',
				input,
				gui,
				tokenInfos,
				i,
				inputVaultIds
			})
		),

		...allTokenOutputs.map(
			(output, i): TokenOutputStep => ({
				type: 'tokenOutput',
				output,
				gui,
				tokenInfos,
				i,
				outputVaultIds
			})
		)
	];

	return deploymentSteps;
};
