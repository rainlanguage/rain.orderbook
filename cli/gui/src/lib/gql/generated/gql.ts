/* eslint-disable */
import * as types from './graphql';
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 */
const documents = {
    "query ordersQuery ($filters: Order_filter) {\n    orders (where: $filters, orderBy: timestamp, orderDirection: desc){\n        id\n        orderHash\n        owner { id }\n        orderJSONString\n        orderActive\n        timestamp\n        expression\n        validInputs {\n            vaultId\n            token {\n                id\n            }\n            tokenVault {\n                id\n                balance\n                balanceDisplay\n                token {\n                    name\n                    decimals\n                    symbol\n                }\n            }\n        }\n        validOutputs {\n            vaultId\n            token {\n                id\n            }\n            tokenVault {\n                id\n                balance\n                balanceDisplay\n                token {\n                    name\n                    decimals\n                    symbol\n                }\n            }\n        }\n        takeOrders {\n            outputIOIndex\n            inputIOIndex\n            input\n            output\n            inputDisplay\n            outputDisplay\n            inputToken {\n                decimals\n                id\n                name\n                symbol\n            }\n            outputToken {\n                decimals\n                id\n                name\n                symbol\n            }\n            sender {\n                id\n            }\n            timestamp\n            transaction {\n                blockNumber\n                timestamp\n                id\n            }\n            id\n        }\n    }\n  }": types.OrdersQueryDocument,
    "query takeOrderEntitiesDynamicFilter ($filters: TakeOrderEntity_filter) {\n    takeOrderEntities (where: $filters, orderBy: timestamp, orderDirection: desc) {\n\t\tid\n\t\tinput\n\t\tinputDisplay\n\t\toutput\n\t\toutputDisplay\n\t\ttimestamp\n\t\torder {\n\t\t\torderHash\n\t\t\tid\n\t\t\towner {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t\tinputToken {\n\t\t\tid\n\t\t\tname\n\t\t\tsymbol\n\t\t\tdecimals\n\t\t}\n\t\toutputToken {\n\t\t\tid\n\t\t\tname\n\t\t\tsymbol\n\t\t\tdecimals\n\t\t}\n\t\tsender {\n\t\t\tid\n\t\t}\n\t\ttransaction {\n\t\t\ttimestamp\n\t\t\tid\n\t\t}\n    }\n  }": types.TakeOrderEntitiesDynamicFilterDocument,
    "query tokenVaults ($filters: TokenVault_filter) {\n    tokenVaults (where: $filters) {\n        vaultId\n        orders {\n            id\n            orderHash\n            orderActive\n            expression\n            expressionDeployer\n        }\n        owner {\n            id \n        }\n        balance\n        balanceDisplay\n        id\n        token {\n            symbol\n            name\n            decimals\n            id\n        }    \n    }\n  }": types.TokenVaultsDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query ordersQuery ($filters: Order_filter) {\n    orders (where: $filters, orderBy: timestamp, orderDirection: desc){\n        id\n        orderHash\n        owner { id }\n        orderJSONString\n        orderActive\n        timestamp\n        expression\n        validInputs {\n            vaultId\n            token {\n                id\n            }\n            tokenVault {\n                id\n                balance\n                balanceDisplay\n                token {\n                    name\n                    decimals\n                    symbol\n                }\n            }\n        }\n        validOutputs {\n            vaultId\n            token {\n                id\n            }\n            tokenVault {\n                id\n                balance\n                balanceDisplay\n                token {\n                    name\n                    decimals\n                    symbol\n                }\n            }\n        }\n        takeOrders {\n            outputIOIndex\n            inputIOIndex\n            input\n            output\n            inputDisplay\n            outputDisplay\n            inputToken {\n                decimals\n                id\n                name\n                symbol\n            }\n            outputToken {\n                decimals\n                id\n                name\n                symbol\n            }\n            sender {\n                id\n            }\n            timestamp\n            transaction {\n                blockNumber\n                timestamp\n                id\n            }\n            id\n        }\n    }\n  }"): (typeof documents)["query ordersQuery ($filters: Order_filter) {\n    orders (where: $filters, orderBy: timestamp, orderDirection: desc){\n        id\n        orderHash\n        owner { id }\n        orderJSONString\n        orderActive\n        timestamp\n        expression\n        validInputs {\n            vaultId\n            token {\n                id\n            }\n            tokenVault {\n                id\n                balance\n                balanceDisplay\n                token {\n                    name\n                    decimals\n                    symbol\n                }\n            }\n        }\n        validOutputs {\n            vaultId\n            token {\n                id\n            }\n            tokenVault {\n                id\n                balance\n                balanceDisplay\n                token {\n                    name\n                    decimals\n                    symbol\n                }\n            }\n        }\n        takeOrders {\n            outputIOIndex\n            inputIOIndex\n            input\n            output\n            inputDisplay\n            outputDisplay\n            inputToken {\n                decimals\n                id\n                name\n                symbol\n            }\n            outputToken {\n                decimals\n                id\n                name\n                symbol\n            }\n            sender {\n                id\n            }\n            timestamp\n            transaction {\n                blockNumber\n                timestamp\n                id\n            }\n            id\n        }\n    }\n  }"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query takeOrderEntitiesDynamicFilter ($filters: TakeOrderEntity_filter) {\n    takeOrderEntities (where: $filters, orderBy: timestamp, orderDirection: desc) {\n\t\tid\n\t\tinput\n\t\tinputDisplay\n\t\toutput\n\t\toutputDisplay\n\t\ttimestamp\n\t\torder {\n\t\t\torderHash\n\t\t\tid\n\t\t\towner {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t\tinputToken {\n\t\t\tid\n\t\t\tname\n\t\t\tsymbol\n\t\t\tdecimals\n\t\t}\n\t\toutputToken {\n\t\t\tid\n\t\t\tname\n\t\t\tsymbol\n\t\t\tdecimals\n\t\t}\n\t\tsender {\n\t\t\tid\n\t\t}\n\t\ttransaction {\n\t\t\ttimestamp\n\t\t\tid\n\t\t}\n    }\n  }"): (typeof documents)["query takeOrderEntitiesDynamicFilter ($filters: TakeOrderEntity_filter) {\n    takeOrderEntities (where: $filters, orderBy: timestamp, orderDirection: desc) {\n\t\tid\n\t\tinput\n\t\tinputDisplay\n\t\toutput\n\t\toutputDisplay\n\t\ttimestamp\n\t\torder {\n\t\t\torderHash\n\t\t\tid\n\t\t\towner {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t\tinputToken {\n\t\t\tid\n\t\t\tname\n\t\t\tsymbol\n\t\t\tdecimals\n\t\t}\n\t\toutputToken {\n\t\t\tid\n\t\t\tname\n\t\t\tsymbol\n\t\t\tdecimals\n\t\t}\n\t\tsender {\n\t\t\tid\n\t\t}\n\t\ttransaction {\n\t\t\ttimestamp\n\t\t\tid\n\t\t}\n    }\n  }"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query tokenVaults ($filters: TokenVault_filter) {\n    tokenVaults (where: $filters) {\n        vaultId\n        orders {\n            id\n            orderHash\n            orderActive\n            expression\n            expressionDeployer\n        }\n        owner {\n            id \n        }\n        balance\n        balanceDisplay\n        id\n        token {\n            symbol\n            name\n            decimals\n            id\n        }    \n    }\n  }"): (typeof documents)["query tokenVaults ($filters: TokenVault_filter) {\n    tokenVaults (where: $filters) {\n        vaultId\n        orders {\n            id\n            orderHash\n            orderActive\n            expression\n            expressionDeployer\n        }\n        owner {\n            id \n        }\n        balance\n        balanceDisplay\n        id\n        token {\n            symbol\n            name\n            decimals\n            id\n        }    \n    }\n  }"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;