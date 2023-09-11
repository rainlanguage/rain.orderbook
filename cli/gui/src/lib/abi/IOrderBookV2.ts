export const IOrderBookV2 =
  [
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "contract IExpressionDeployerV1",
          "name": "expressionDeployer",
          "type": "address"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "indexed": false,
          "internalType": "struct Order",
          "name": "order",
          "type": "tuple"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "orderHash",
          "type": "uint256"
        }
      ],
      "name": "AddOrder",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "components": [
            {
              "internalType": "uint256",
              "name": "aliceOutput",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobOutput",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "aliceInput",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobInput",
              "type": "uint256"
            }
          ],
          "indexed": false,
          "internalType": "struct ClearStateChange",
          "name": "clearStateChange",
          "type": "tuple"
        }
      ],
      "name": "AfterClear",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "indexed": false,
          "internalType": "struct Order",
          "name": "alice",
          "type": "tuple"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "indexed": false,
          "internalType": "struct Order",
          "name": "bob",
          "type": "tuple"
        },
        {
          "components": [
            {
              "internalType": "uint256",
              "name": "aliceInputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "aliceOutputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobInputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobOutputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "aliceBountyVaultId",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobBountyVaultId",
              "type": "uint256"
            }
          ],
          "indexed": false,
          "internalType": "struct ClearConfig",
          "name": "clearConfig",
          "type": "tuple"
        }
      ],
      "name": "Clear",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256[][]",
          "name": "context",
          "type": "uint256[][]"
        }
      ],
      "name": "Context",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "token",
              "type": "address"
            },
            {
              "internalType": "uint256",
              "name": "vaultId",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "amount",
              "type": "uint256"
            }
          ],
          "indexed": false,
          "internalType": "struct DepositConfig",
          "name": "config",
          "type": "tuple"
        }
      ],
      "name": "Deposit",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "orderHash",
          "type": "uint256"
        }
      ],
      "name": "OrderExceedsMaxRatio",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "orderHash",
          "type": "uint256"
        }
      ],
      "name": "OrderNotFound",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "orderHash",
          "type": "uint256"
        }
      ],
      "name": "OrderZeroAmount",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "indexed": false,
          "internalType": "struct Order",
          "name": "order",
          "type": "tuple"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "orderHash",
          "type": "uint256"
        }
      ],
      "name": "RemoveOrder",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "components": [
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "owner",
                  "type": "address"
                },
                {
                  "internalType": "bool",
                  "name": "handleIO",
                  "type": "bool"
                },
                {
                  "components": [
                    {
                      "internalType": "contract IInterpreterV1",
                      "name": "interpreter",
                      "type": "address"
                    },
                    {
                      "internalType": "contract IInterpreterStoreV1",
                      "name": "store",
                      "type": "address"
                    },
                    {
                      "internalType": "address",
                      "name": "expression",
                      "type": "address"
                    }
                  ],
                  "internalType": "struct Evaluable",
                  "name": "evaluable",
                  "type": "tuple"
                },
                {
                  "components": [
                    {
                      "internalType": "address",
                      "name": "token",
                      "type": "address"
                    },
                    {
                      "internalType": "uint8",
                      "name": "decimals",
                      "type": "uint8"
                    },
                    {
                      "internalType": "uint256",
                      "name": "vaultId",
                      "type": "uint256"
                    }
                  ],
                  "internalType": "struct IO[]",
                  "name": "validInputs",
                  "type": "tuple[]"
                },
                {
                  "components": [
                    {
                      "internalType": "address",
                      "name": "token",
                      "type": "address"
                    },
                    {
                      "internalType": "uint8",
                      "name": "decimals",
                      "type": "uint8"
                    },
                    {
                      "internalType": "uint256",
                      "name": "vaultId",
                      "type": "uint256"
                    }
                  ],
                  "internalType": "struct IO[]",
                  "name": "validOutputs",
                  "type": "tuple[]"
                }
              ],
              "internalType": "struct Order",
              "name": "order",
              "type": "tuple"
            },
            {
              "internalType": "uint256",
              "name": "inputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "outputIOIndex",
              "type": "uint256"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "signer",
                  "type": "address"
                },
                {
                  "internalType": "uint256[]",
                  "name": "context",
                  "type": "uint256[]"
                },
                {
                  "internalType": "bytes",
                  "name": "signature",
                  "type": "bytes"
                }
              ],
              "internalType": "struct SignedContextV1[]",
              "name": "signedContext",
              "type": "tuple[]"
            }
          ],
          "indexed": false,
          "internalType": "struct TakeOrderConfig",
          "name": "config",
          "type": "tuple"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "input",
          "type": "uint256"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "output",
          "type": "uint256"
        }
      ],
      "name": "TakeOrder",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": false,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "token",
              "type": "address"
            },
            {
              "internalType": "uint256",
              "name": "vaultId",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "amount",
              "type": "uint256"
            }
          ],
          "indexed": false,
          "internalType": "struct WithdrawConfig",
          "name": "config",
          "type": "tuple"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "amount",
          "type": "uint256"
        }
      ],
      "name": "Withdraw",
      "type": "event"
    },
    {
      "inputs": [
        {
          "components": [
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "contract IExpressionDeployerV1",
                  "name": "deployer",
                  "type": "address"
                },
                {
                  "internalType": "bytes[]",
                  "name": "sources",
                  "type": "bytes[]"
                },
                {
                  "internalType": "uint256[]",
                  "name": "constants",
                  "type": "uint256[]"
                }
              ],
              "internalType": "struct EvaluableConfig",
              "name": "evaluableConfig",
              "type": "tuple"
            },
            {
              "internalType": "bytes",
              "name": "meta",
              "type": "bytes"
            }
          ],
          "internalType": "struct OrderConfig",
          "name": "config",
          "type": "tuple"
        }
      ],
      "name": "addOrder",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "internalType": "struct Order",
          "name": "alice",
          "type": "tuple"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "internalType": "struct Order",
          "name": "bob",
          "type": "tuple"
        },
        {
          "components": [
            {
              "internalType": "uint256",
              "name": "aliceInputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "aliceOutputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobInputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobOutputIOIndex",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "aliceBountyVaultId",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "bobBountyVaultId",
              "type": "uint256"
            }
          ],
          "internalType": "struct ClearConfig",
          "name": "clearConfig",
          "type": "tuple"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "signer",
              "type": "address"
            },
            {
              "internalType": "uint256[]",
              "name": "context",
              "type": "uint256[]"
            },
            {
              "internalType": "bytes",
              "name": "signature",
              "type": "bytes"
            }
          ],
          "internalType": "struct SignedContextV1[]",
          "name": "aliceSignedContext",
          "type": "tuple[]"
        },
        {
          "components": [
            {
              "internalType": "address",
              "name": "signer",
              "type": "address"
            },
            {
              "internalType": "uint256[]",
              "name": "context",
              "type": "uint256[]"
            },
            {
              "internalType": "bytes",
              "name": "signature",
              "type": "bytes"
            }
          ],
          "internalType": "struct SignedContextV1[]",
          "name": "bobSignedContext",
          "type": "tuple[]"
        }
      ],
      "name": "clear",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "components": [
            {
              "internalType": "address",
              "name": "token",
              "type": "address"
            },
            {
              "internalType": "uint256",
              "name": "vaultId",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "amount",
              "type": "uint256"
            }
          ],
          "internalType": "struct DepositConfig",
          "name": "config",
          "type": "tuple"
        }
      ],
      "name": "deposit",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "token",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "amount",
          "type": "uint256"
        }
      ],
      "name": "flashFee",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "contract IERC3156FlashBorrower",
          "name": "receiver",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "token",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "amount",
          "type": "uint256"
        },
        {
          "internalType": "bytes",
          "name": "data",
          "type": "bytes"
        }
      ],
      "name": "flashLoan",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "token",
          "type": "address"
        }
      ],
      "name": "maxFlashLoan",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "components": [
            {
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "internalType": "bool",
              "name": "handleIO",
              "type": "bool"
            },
            {
              "components": [
                {
                  "internalType": "contract IInterpreterV1",
                  "name": "interpreter",
                  "type": "address"
                },
                {
                  "internalType": "contract IInterpreterStoreV1",
                  "name": "store",
                  "type": "address"
                },
                {
                  "internalType": "address",
                  "name": "expression",
                  "type": "address"
                }
              ],
              "internalType": "struct Evaluable",
              "name": "evaluable",
              "type": "tuple"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validInputs",
              "type": "tuple[]"
            },
            {
              "components": [
                {
                  "internalType": "address",
                  "name": "token",
                  "type": "address"
                },
                {
                  "internalType": "uint8",
                  "name": "decimals",
                  "type": "uint8"
                },
                {
                  "internalType": "uint256",
                  "name": "vaultId",
                  "type": "uint256"
                }
              ],
              "internalType": "struct IO[]",
              "name": "validOutputs",
              "type": "tuple[]"
            }
          ],
          "internalType": "struct Order",
          "name": "order",
          "type": "tuple"
        }
      ],
      "name": "removeOrder",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "components": [
            {
              "internalType": "address",
              "name": "output",
              "type": "address"
            },
            {
              "internalType": "address",
              "name": "input",
              "type": "address"
            },
            {
              "internalType": "uint256",
              "name": "minimumInput",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "maximumInput",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "maximumIORatio",
              "type": "uint256"
            },
            {
              "components": [
                {
                  "components": [
                    {
                      "internalType": "address",
                      "name": "owner",
                      "type": "address"
                    },
                    {
                      "internalType": "bool",
                      "name": "handleIO",
                      "type": "bool"
                    },
                    {
                      "components": [
                        {
                          "internalType": "contract IInterpreterV1",
                          "name": "interpreter",
                          "type": "address"
                        },
                        {
                          "internalType": "contract IInterpreterStoreV1",
                          "name": "store",
                          "type": "address"
                        },
                        {
                          "internalType": "address",
                          "name": "expression",
                          "type": "address"
                        }
                      ],
                      "internalType": "struct Evaluable",
                      "name": "evaluable",
                      "type": "tuple"
                    },
                    {
                      "components": [
                        {
                          "internalType": "address",
                          "name": "token",
                          "type": "address"
                        },
                        {
                          "internalType": "uint8",
                          "name": "decimals",
                          "type": "uint8"
                        },
                        {
                          "internalType": "uint256",
                          "name": "vaultId",
                          "type": "uint256"
                        }
                      ],
                      "internalType": "struct IO[]",
                      "name": "validInputs",
                      "type": "tuple[]"
                    },
                    {
                      "components": [
                        {
                          "internalType": "address",
                          "name": "token",
                          "type": "address"
                        },
                        {
                          "internalType": "uint8",
                          "name": "decimals",
                          "type": "uint8"
                        },
                        {
                          "internalType": "uint256",
                          "name": "vaultId",
                          "type": "uint256"
                        }
                      ],
                      "internalType": "struct IO[]",
                      "name": "validOutputs",
                      "type": "tuple[]"
                    }
                  ],
                  "internalType": "struct Order",
                  "name": "order",
                  "type": "tuple"
                },
                {
                  "internalType": "uint256",
                  "name": "inputIOIndex",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "outputIOIndex",
                  "type": "uint256"
                },
                {
                  "components": [
                    {
                      "internalType": "address",
                      "name": "signer",
                      "type": "address"
                    },
                    {
                      "internalType": "uint256[]",
                      "name": "context",
                      "type": "uint256[]"
                    },
                    {
                      "internalType": "bytes",
                      "name": "signature",
                      "type": "bytes"
                    }
                  ],
                  "internalType": "struct SignedContextV1[]",
                  "name": "signedContext",
                  "type": "tuple[]"
                }
              ],
              "internalType": "struct TakeOrderConfig[]",
              "name": "orders",
              "type": "tuple[]"
            }
          ],
          "internalType": "struct TakeOrdersConfig",
          "name": "config",
          "type": "tuple"
        }
      ],
      "name": "takeOrders",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "totalInput",
          "type": "uint256"
        },
        {
          "internalType": "uint256",
          "name": "totalOutput",
          "type": "uint256"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "token",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "id",
          "type": "uint256"
        }
      ],
      "name": "vaultBalance",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "balance",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "components": [
            {
              "internalType": "address",
              "name": "token",
              "type": "address"
            },
            {
              "internalType": "uint256",
              "name": "vaultId",
              "type": "uint256"
            },
            {
              "internalType": "uint256",
              "name": "amount",
              "type": "uint256"
            }
          ],
          "internalType": "struct WithdrawConfig",
          "name": "config",
          "type": "tuple"
        }
      ],
      "name": "withdraw",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    }
  ] as const