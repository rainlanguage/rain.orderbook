import { expect, test } from 'vitest'
import type { Dictionary } from 'lodash';
import { pickDeployments, pickScenarios } from '$lib/services/pickConfig';
import type { Config, ConfigSource, DeploymentConfigSource, Scenario } from '$lib/typeshare/config';

export const mockedConfig: Config = {
  "networks": {
    "flare": {
      "name": "flare",
      "rpc": "https://rpc.ankr.com/flare",
      "chain-id": 14,
      "label": "Flare",
      "network-id": 14,
      "currency": "FLR"
    },
    "polygon": {
      "name": "flare",
      "rpc": "https://rpc.ankr.com/polygon",
      "chain-id": 137,
      "label": "Polygon",
      "network-id": 137,
      "currency": "MATIC"
    }
  },
  "subgraphs": {
    "flare": "https://subgraphs.h20liquidity.tech/subgraphs/name/flare-ob"
  },
  "orderbooks": {
    "flare": {
      "address": "0xf9bdedb1e8c32185e879e056eba9f5aec1839d60",
      "network": {
        "name": "flare",
        "rpc": "https://rpc.ankr.com/flare",
        "chain-id": 14,
        "label": "Flare",
        "network-id": 14,
        "currency": "FLR"
      },
      "subgraph": "https://subgraphs.h20liquidity.tech/subgraphs/name/flare-ob",
      "label": "Flare Orderbook"
    }
  },
  "tokens": {
    "wflr": {
      "network": {
        "name": "flare",
        "rpc": "https://rpc.ankr.com/flare",
        "chain-id": 14,
        "label": "Flare",
        "network-id": 14,
        "currency": "FLR"
      },
      "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    },
    "weth": {
      "network": {
        "name": "flare",
        "rpc": "https://rpc.ankr.com/flare",
        "chain-id": 14,
        "label": "Flare",
        "network-id": 14,
        "currency": "FLR"
      },
      "address": "0x62bd084cbcd6c85347c50292a141ea4d3e7e3511",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    }
  },
  "deployers": {
    "flare": {
      "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
      "network": {
        "name": "flare",
        "rpc": "https://rpc.ankr.com/flare",
        "chain-id": 14,
        "label": "Flare",
        "network-id": 14,
        "currency": "FLR"
      },
      "label": "Flare Deployer"
    }
  },
  "orders": {
    "buy-wflr": {
      "inputs": [
        {
          "token": {
            "network": {
              "name": "flare",
              "rpc": "https://rpc.ankr.com/flare",
              "chain-id": 14,
              "label": "Flare",
              "network-id": 14,
              "currency": "FLR"
            },
            "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
        }
      ],
      "outputs": [
        {
          "token": {
            "network": {
              "name": "flare",
              "rpc": "https://rpc.ankr.com/flare",
              "chain-id": 14,
              "label": "Flare",
              "network-id": 14,
              "currency": "FLR"
            },
            "address": "0x62bd084cbcd6c85347c50292a141ea4d3e7e3511",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
        }
      ],
      "network": {
        "name": "flare",
        "rpc": "https://rpc.ankr.com/flare",
        "chain-id": 14,
        "label": "Flare",
        "network-id": 14,
        "currency": "FLR"
      },
      "deployer": undefined,
      "orderbook": undefined
    },
    "sell-wflr": {
      "inputs": [
        {
          "token": {
            "network": {
              "name": "flare",
              "rpc": "https://rpc.ankr.com/flare",
              "chain-id": 14,
              "label": "Flare",
              "network-id": 14,
              "currency": "FLR"
            },
            "address": "0x62bd084cbcd6c85347c50292a141ea4d3e7e3511",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
        }
      ],
      "outputs": [
        {
          "token": {
            "network": {
              "name": "flare",
              "rpc": "https://rpc.ankr.com/flare",
              "chain-id": 14,
              "label": "Flare",
              "network-id": 14,
              "currency": "FLR"
            },
            "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
        }
      ],
      "network": {
        "name": "flare",
        "rpc": "https://rpc.ankr.com/flare",
        "chain-id": 14,
        "label": "Flare",
        "network-id": 14,
        "currency": "FLR"
      },
      "deployer": undefined,
      "orderbook": undefined
    }
  },
  "scenarios": {
    "flare.sell-wflr": {
      "name": "flare.sell-wflr",
      "bindings": {
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
        "ftso-base": "\"FLR\"",
        "ftso-quote": "\"ETH\"",
        "spread-multiplier": "101e16"
      },
      "runs": 1,
      "deployer": {
        "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "label": "Flare Deployer"
      }
    },
    "flare": {
      "name": "flare",
      "bindings": {
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
        "spread-multiplier": "101e16"
      },
      "runs": undefined,
      "deployer": {
        "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "label": "Flare Deployer"
      }
    },
    "flare.buy-wflr": {
      "name": "flare.buy-wflr",
      "bindings": {
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
        "ftso-base": "\"ETH\"",
        "ftso-quote": "\"FLR\"",
        "spread-multiplier": "101e16"
      },
      "runs": undefined,
      "deployer": {
        "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "label": "Flare Deployer"
      }
    }
  },
  "charts": {},
  "deployments": {
    "sell-wflr": {
      "scenario": {
        "name": "flare.sell-wflr",
        "bindings": {
          "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
          "ftso-base": "\"FLR\"",
          "ftso-quote": "\"ETH\"",
          "spread-multiplier": "101e16"
        },
        "runs": 1,
        "deployer": {
          "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
          "network": {
            "name": "flare",
            "rpc": "https://rpc.ankr.com/flare",
            "chain-id": 14,
            "label": "Flare",
            "network-id": 14,
            "currency": "FLR"
          },
          "label": "Flare Deployer"
        }
      },
      "order": {
        "inputs": [
          {
            "token": {
              "network": {
                "name": "flare",
                "rpc": "https://rpc.ankr.com/flare",
                "chain-id": 14,
                "label": "Flare",
                "network-id": 14,
                "currency": "FLR"
              },
              "address": "0x62bd084cbcd6c85347c50292a141ea4d3e7e3511",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
          }
        ],
        "outputs": [
          {
            "token": {
              "network": {
                "name": "flare",
                "rpc": "https://rpc.ankr.com/flare",
                "chain-id": 14,
                "label": "Flare",
                "network-id": 14,
                "currency": "FLR"
              },
              "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
          }
        ],
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "deployer": undefined,
        "orderbook": undefined
      }
    },
    "buy-wflr": {
      "scenario": {
        "name": "flare.buy-wflr",
        "bindings": {
          "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
          "ftso-base": "\"ETH\"",
          "ftso-quote": "\"FLR\"",
          "spread-multiplier": "101e16"
        },
        "runs": undefined,
        "deployer": {
          "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
          "network": {
            "name": "flare",
            "rpc": "https://rpc.ankr.com/flare",
            "chain-id": 14,
            "label": "Flare",
            "network-id": 14,
            "currency": "FLR"
          },
          "label": "Flare Deployer"
        }
      },
      "order": {
        "inputs": [
          {
            "token": {
              "network": {
                "name": "flare",
                "rpc": "https://rpc.ankr.com/flare",
                "chain-id": 14,
                "label": "Flare",
                "network-id": 14,
                "currency": "FLR"
              },
              "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
          }
        ],
        "outputs": [
          {
            "token": {
              "network": {
                "name": "flare",
                "rpc": "https://rpc.ankr.com/flare",
                "chain-id": 14,
                "label": "Flare",
                "network-id": 14,
                "currency": "FLR"
              },
              "address": "0x62bd084cbcd6c85347c50292a141ea4d3e7e3511",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20c"
          }
        ],
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "deployer": undefined,
        "orderbook": undefined
      }
    }
  },
  "sentry": undefined
}


export const mockedConfigSource: ConfigSource = {
  "networks": {
    "flare": {
      "rpc": "https://rpc.ankr.com/flare",
      "chain-id": 14,
      "label": "Flare",
      "network-id": 14,
      "currency": "FLR"
    },
    "polygon": {
      "rpc": "https://rpc.ankr.com/polygon",
      "chain-id": 137,
      "label": "Polygon",
      "network-id": 137,
      "currency": "MATIC"
    }
  },
  "subgraphs": {
    "flare": "https://subgraphs.h20liquidity.tech/subgraphs/name/flare-ob"
  },
  "orderbooks": {
    "flare": {
      "address": "0xf9bdedb1e8c32185e879e056eba9f5aec1839d60",
      "network": "flare",
      "subgraph": "flare",
      "label": "Flare Orderbook"
    }
  },
  "tokens": {
    "wflr": {
      "network": "flare",
      "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    },
    "weth": {
      "network": "flare",
      "address": "0x62bd084cbcd6c85347c50292a141ea4d3e7e3511",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    }
  },
  "deployers": {
    "flare": {
      "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
      "network": "flare",
      "label": "Flare Deployer"
    }
  },
  "orders": {
    "sell-wflr": {
      "inputs": [
        {
          "token": "weth",
          "vault-id": 0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20cn
        }
      ],
      "outputs": [
        {
          "token": "wflr",
          "vault-id": 0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20cn
        }
      ],
      "deployer": undefined,
      "orderbook": undefined
    },
    "buy-wflr": {
      "inputs": [
        {
          "token": "wflr",
          "vault-id": 0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20cn
        }
      ],
      "outputs": [
        {
          "token": "weth",
          "vault-id": 0xcec26d26ca191a871ae3153b1f3d67c6bb2b00fd0a063db10ea8e8b0d524f20cn
        }
      ],
      "deployer": undefined,
      "orderbook": undefined
    }
  },
  "scenarios": {
    "flare": {
      "bindings": {
        "spread-multiplier": "101e16",
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB"
      },
      "runs": undefined,
      "deployer": undefined,
      "scenarios": {
        "buy-wflr": {
          "bindings": {
            "ftso-quote": "\"FLR\"",
            "ftso-base": "\"ETH\""
          },
          "runs": undefined,
          "deployer": undefined,
          "scenarios": undefined
        },
        "sell-wflr": {
          "bindings": {
            "ftso-quote": "\"ETH\"",
            "ftso-base": "\"FLR\""
          },
          "runs": 1,
          "deployer": undefined,
          "scenarios": undefined
        }
      }
    }
  },
  "charts": {},
  "deployments": {
    "buy-wflr": {
      "scenario": "flare.buy-wflr",
      "order": "buy-wflr"
    },
    "sell-wflr": {
      "scenario": "flare.sell-wflr",
      "order": "sell-wflr"
    }
  },
  "sentry": undefined
}

test('pick deployments', () => {
  const activeNetwork = "flare";
  const result = pickDeployments(mockedConfigSource, mockedConfig, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {
    "sell-wflr": {
      "scenario": "flare.sell-wflr",
      "order": "sell-wflr"
    },
    "buy-wflr": {
      "scenario": "flare.buy-wflr",
      "order": "buy-wflr"
    }
  };

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick deployments when empty', () => {
  const activeNetwork = "polygon";
  const result = pickDeployments(mockedConfigSource, mockedConfig, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {};

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick scenarios', () => {
  const activeNetwork = "flare";
  const result = pickScenarios(mockedConfig, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {
    "flare.sell-wflr": {
      "name": "flare.sell-wflr",
      "bindings": {
        "ftso-base": "\"FLR\"",
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
        "ftso-quote": "\"ETH\"",
        "spread-multiplier": "101e16"
      },
      "runs": 1,
      "deployer": {
        "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "label": "Flare Deployer"
      }
    },
    "flare.buy-wflr": {
      "name": "flare.buy-wflr",
      "bindings": {
        "ftso-base": "\"ETH\"",
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
        "ftso-quote": "\"FLR\"",
        "spread-multiplier": "101e16"
      },
      "runs": undefined,
      "deployer": {
        "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "label": "Flare Deployer"
      }
    },
    "flare": {
      "name": "flare",
      "bindings": {
        "flare-sub-parser": "0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB",
        "spread-multiplier": "101e16"
      },
      "runs": undefined,
      "deployer": {
        "address": "0xd19581a021f4704ad4ebff68258e7a0a9db1cd77",
        "network": {
          "name": "flare",
          "rpc": "https://rpc.ankr.com/flare",
          "chain-id": 14,
          "label": "Flare",
          "network-id": 14,
          "currency": "FLR"
        },
        "label": "Flare Deployer"
      }
    }
  };

  expect(result).toStrictEqual(expectedPickedScenarios);
});

test('pick scenarios when empty', () => {
  const activeNetwork = "polygon";
  const result = pickScenarios(mockedConfig, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {};

  expect(result).toStrictEqual(expectedPickedScenarios);
});