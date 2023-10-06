use ethers::prelude::abigen;

abigen!(
  RainterpreterExpressionDeployer,
  "tests/utils/deploy/touch_deployer/RainterpreterExpressionDeployer.json",
  derives(serde::Deserialize, serde::Serialize);

  Rainterpreter,
  "tests/utils/deploy/touch_deployer/Rainterpreter.json";

  RainterpreterStore,
  "tests/utils/deploy/touch_deployer/RainterpreterStore.json";

  AuthoringMetaGetter,
  "tests/utils/deploy/touch_deployer/AuthoringMetaGetter.json"
);
