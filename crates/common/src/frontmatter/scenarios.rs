use dotrain::Rebind;
use strict_yaml_rust::{scanner::ScanError, StrictYamlLoader};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct MontecarloScenario {
    pub name: String,
    pub runs: u64,
    pub binds: Vec<Rebind>,
    pub fuzz_binds: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegularScenario {
    pub name: String,
    pub binds: Vec<Rebind>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Scenario {
    Montecarlo(MontecarloScenario),
    Regular(RegularScenario),
}

#[derive(Error, Debug, PartialEq)]
pub enum ScenarioParsingError {
    #[error("YAML parsing error")]
    YamlParsingError(#[from] ScanError),

    #[error("No scenarios")]
    NoScenarios,

    #[error("No name")]
    NoName,

    #[error("No binds")]
    NoBinds,

    #[error("Invalid scenario configuration")]
    InvalidScenarioConfig,
}

impl Scenario {
    pub fn name(&self) -> &str {
        match self {
            Scenario::Montecarlo(mc) => &mc.name,
            Scenario::Regular(reg) => &reg.name,
        }
    }

    pub fn binds(&self) -> Vec<Rebind> {
        match self {
            Scenario::Montecarlo(mc) => mc.binds.clone(),
            Scenario::Regular(reg) => reg.binds.clone(),
        }
    }

    pub fn fuzz_binds(&self) -> Option<&Vec<String>> {
        if let Scenario::Montecarlo(mc) = self {
            Some(&mc.fuzz_binds)
        } else {
            None
        }
    }

    pub fn runs(&self) -> Option<u64> {
        if let Scenario::Montecarlo(mc) = self {
            Some(mc.runs)
        } else {
            None
        }
    }

    pub fn parse_scenarios(frontmatter: &str) -> Result<Vec<Scenario>, ScenarioParsingError> {
        let frontmatter_yaml_vec = StrictYamlLoader::load_from_str(frontmatter)
            .map_err(|_| ScenarioParsingError::InvalidScenarioConfig)?;
        let frontmatter_yaml = frontmatter_yaml_vec
            .first()
            .ok_or(ScenarioParsingError::NoScenarios)?;

        let scenarios_yaml = frontmatter_yaml["scenarios"]
            .as_hash()
            .ok_or(ScenarioParsingError::NoScenarios)?;

        let mut scenarios: Vec<Scenario> = Vec::new();

        for (name_yaml, scenario_yaml) in scenarios_yaml {
            let name = name_yaml
                .as_str()
                .ok_or(ScenarioParsingError::NoName)?
                .to_string();

            let binds_yaml = scenario_yaml["bind"]
                .as_vec()
                .ok_or(ScenarioParsingError::NoBinds)?;

            let mut binds: Vec<Rebind> = Vec::new();
            let mut fuzz_binds: Vec<String> = Vec::new();

            for bind_yaml in binds_yaml {
                if let Some(hash) = bind_yaml.as_hash() {
                    for (key, value) in hash {
                        let key_str = key
                            .as_str()
                            .ok_or(ScenarioParsingError::InvalidScenarioConfig)?;
                        let value_str = value
                            .as_str()
                            .ok_or(ScenarioParsingError::InvalidScenarioConfig)?;
                        binds.push(Rebind(key_str.to_string(), value_str.to_string()));
                    }
                } else if let Some(str) = bind_yaml.as_str() {
                    fuzz_binds.push(str.to_string());
                } else {
                    return Err(ScenarioParsingError::InvalidScenarioConfig);
                }
            }

            if let Some(runs_yaml) = scenario_yaml["runs"].as_str() {
                let runs = runs_yaml
                    .parse::<u64>()
                    .map_err(|_| ScenarioParsingError::InvalidScenarioConfig)?;
                scenarios.push(Scenario::Montecarlo(MontecarloScenario {
                    name,
                    runs,
                    binds,
                    fuzz_binds,
                }));
            } else if fuzz_binds.is_empty() {
                scenarios.push(Scenario::Regular(RegularScenario { name, binds }));
            } else {
                return Err(ScenarioParsingError::InvalidScenarioConfig); // Montecarlo scenario without runs or regular scenario with fuzz_binds is invalid
            }
        }

        Ok(scenarios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scenarios() {
        let frontmatter = r#"
scenarios:
    main:
        bind:
        - some-binding: 12345
    test:
        runs: 100
        bind:
        - bound: 123
        - to-be-fuzzed
        - some-binding"#;

        let scenarios = Scenario::parse_scenarios(frontmatter);

        assert!(scenarios.is_ok());

        let scenarios = scenarios.unwrap();

        assert_eq!(scenarios.len(), 2);

        let main_scenario = scenarios.get(0).unwrap();
        assert_eq!(
            main_scenario,
            &Scenario::Regular(RegularScenario {
                name: "main".to_string(),
                binds: vec![Rebind("some-binding".to_string(), "12345".to_string())],
            })
        );

        let test_scenario = scenarios.get(1).unwrap();
        assert_eq!(
            test_scenario,
            &Scenario::Montecarlo(MontecarloScenario {
                name: "test".to_string(),
                runs: 100,
                binds: vec![Rebind("bound".to_string(), "123".to_string())],
                fuzz_binds: vec!["to-be-fuzzed".to_string(), "some-binding".to_string()],
            })
        );
    }

    #[test]
    fn test_parse_scenarios_invalid() {
        let frontmatter = r#"
scenarios:
    test:
        bind:
        - bound: 123
        - to-be-fuzzed
        - some-binding"#;

        let scenarios = Scenario::parse_scenarios(frontmatter);

        assert_eq!(scenarios, Err(ScenarioParsingError::InvalidScenarioConfig));
    }
}
