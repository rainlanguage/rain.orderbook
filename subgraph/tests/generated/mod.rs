use ethers::prelude::abigen;

abigen!(
  RainterpreterExpressionDeployer,
  "tests/generated/RainterpreterExpressionDeployerNP.json",
  derives(serde::Deserialize, serde::Serialize);

  Rainterpreter,
  "tests/generated/RainterpreterNP.json";

  RainterpreterStore,
  "tests/generated/RainterpreterStore.json";

  AuthoringMetaGetter,
  "tests/generated/AuthoringMetaGetter.json";
);
