import { isHex } from "viem"

export type ExpressionJson = {
    sources?: string[],
    constants?: string[]
}

export type Expression = {
    sources: `0x${string}`[],
    constants: bigint[]
}

export const parseExpressionJson = (orderJson: ExpressionJson): Expression => {
    if (!orderJson?.sources) throw new Error("No sources found in orderJson")

    if (!orderJson?.constants) throw new Error("No constants found in orderJson")
    if (!orderJson.sources.every(source => isHex(source))) throw new Error("Not all sources are hex")
    if (!orderJson.constants.every(constant => isStringAnInteger(constant))) throw new Error("Not all constants are integers")

    const constants = orderJson.constants.map(constant => BigInt(constant));
    const sources = orderJson.sources as `0x${string}`[];
    return {
        constants, sources
    }
}

export const isStringAnInteger = (str: string) => {
    // Convert the string to a number
    const num = Number(str);
    // Check if it's not NaN, it's an integer, and it matches the original string when parsed
    return !isNaN(num) && parseInt(str, 10) === num;
}

export const getExpressionConstantsIndexes = (expression: Expression, constants: bigint[]): number[] => {
    return constants.map(constant => expression.constants.findIndex(constantInExpression => constantInExpression === constant));
}

export const getExpressionConstantsByIndexes = (expression: Expression, indexes: number[]): bigint[] => {
    return indexes.map(index => expression.constants[index]);
}

export const replaceExpressionConstant = (expression: Expression, index: number, newConstant: bigint) => {
    const newExpression = { ...expression };
    newExpression.constants[index] = newConstant;
    return newExpression;
}