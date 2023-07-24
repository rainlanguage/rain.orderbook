use schemars::JsonSchema;
use crate::meta::rain::v1::Operand;
use crate::meta::rain::v1::RainSymbol;
use crate::meta::rain::v1::Description;
use crate::meta::rain::v1::RainString;
use serde::Deserialize;
use serde::Serialize;
use validator::Validate;
use validator::ValidationErrors;
use validator::ValidationError;

pub type Computation = RainString;

/// BitIntegers are zero indexed.
pub const MIN_BIT_INTEGER: usize = 0;
/// BitIntegers cannot range past the size of an Operand in bits, zero indexed.
pub const MAX_BIT_INTEGER: usize = (std::mem::size_of::<Operand>() * 8) - 1;

/// # BitInteger
/// Counts or ranges bits in an operand. Ranges are 0 indexed.
#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
#[serde(transparent)]
pub struct BitInteger{
    #[validate(range(min = "MIN_BIT_INTEGER", max = "MAX_BIT_INTEGER"))]
    pub value: u8
}

/// # BitIntegerRange
#[derive(JsonSchema, Debug, Serialize, Deserialize)]
pub struct BitIntegerRange(BitInteger, BitInteger);

impl Validate for BitIntegerRange {
    fn validate(&self) -> Result<(), ValidationErrors> {
        ValidationErrors::merge_all(
            if self.0 <= self.1 {
                Ok(())
            } else {
                let mut errors = ValidationErrors::new();
                errors.add("range", ValidationError::new("Bad bit integer range.\n"));
                Err(errors)
            },
            "range",
            vec![self.0.validate(), self.1.validate()]
        )
    }
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
pub enum OperandArgRange {
    Exact(Operand),
    Range(Operand, Operand),
}

impl Validate for OperandArgRange {
    fn validate(&self) -> Result<(), ValidationErrors> {
        ValidationErrors::merge_all(
            match self {
                OperandArgRange::Exact(_) => Ok(()),
                OperandArgRange::Range(min, max) => if min <= max {
                    Ok(())
                } else {
                    let mut errors = ValidationErrors::new();
                    errors.add("range", ValidationError::new("Bad operand arg range.\n"));
                    Err(errors)
                }
            },
            "range",
            match self {
                OperandArgRange::Exact(exact) => vec![exact.validate()],
                OperandArgRange::Range(min, max) => vec![min.validate(), max.validate()],
            }
        )
    }
}

/// # OpMeta.
/// Opcodes metadata used by Rainlang.
#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize)]
pub struct OpMeta {
    /// # Name
    /// Primary word used to identify the opcode.
    #[validate]
    pub name: RainSymbol,
    /// # Description
    /// Brief description of the opcode.
    #[serde(default)]
    #[validate]
    pub desc: Description,
    /// # Operand
    /// Data required to calculate and format the operand.
    #[serde(default)]
    #[validate]
    pub operand: Vec<OperandArg>,
    /// # Inputs
    /// Data required to specify the inputs of the opcode. 0 for opcodes with no
    /// input, for opcodes with constant number of inputs, the length of
    /// "parameters" array defines the number of inputs and for opcodes with
    /// dynamic number of inputs, "bits" field must be specified which determines
    /// this opcode has dynamic inputs and number of inputs will be derived from
    /// the operand bits with "computation" field applied if specified.
    #[serde(default)]
    #[validate]
    pub inputs: Vec<Input>,
    /// # Outputs
    /// Data required to specify the outputs of the opcode. An integer specifies
    /// the number of outputs for opcodes with constants number of outputs and
    /// for opcodes with dynamic outputs the "bits" field will determine the
    /// number of outputs with "computation" field applied if specified.
    #[serde(default)]
    #[validate]
    pub outputs: Vec<Output>,
    /// # Aliases
    /// Other words used to reference the opcode.
    #[serde(default)]
    #[validate]
    pub aliases: Vec<RainSymbol>,
}

/// # Input
/// Data type of opcode's inputs that determines the number of inputs an opcode
/// has and provide information about them.
#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize)]
pub struct Input {
    /// # Parameters
    /// List of InputParameters, may be empty.
    #[serde(default)]
    #[validate]
    pub parameters: Vec<InputParameter>,
    /// # Inputs-Allocated Operand Bits
    /// Specifies bits of the operand allocated for number of inputs. Determines
    /// the number of inputs for a computed opcode inputs. Required only for
    /// computed (non-constant) inputs.
    #[serde(default)]
    #[validate]
    pub bits: Option<BitIntegerRange>,
    /// # Inputs-Allocated Operand Bits Computation
    /// Specifies any arithmetical operation that will be applied to the value of
    /// the extracted operand bits. The "bits" keyword is reserved for accessing
    /// the extracted value, example: "(bits + 1) * 2". Required only for
    /// computed (non-constant) inputs.
    #[serde(default)]
    #[validate]
    pub computation: Option<Computation>,
}

/// # Input Parameter
/// Data type for opcode's inputs parameters, the length determines the number of
/// inputs for constant (non-computed) inputs.
#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize)]
pub struct InputParameter {
    /// # Input Parameter Name
    /// Name of the input parameter.
    #[validate]
    pub name: RainSymbol,
    /// # Input Parameter Description
    /// Description of the input parameter.
    #[serde(default)]
    #[validate]
    pub desc: Description,
    /// # Parameter Spread
    /// Specifies if an argument is dynamic in length, default is false, so only
    /// needs to be defined if an argument is spread.
    #[serde(default)]
    pub spread: bool,
}

/// # Output
/// Data type of opcode's outputs that determines the number of outputs an opcode
/// has and provide information about them.
#[derive(JsonSchema, Debug, Serialize, Deserialize)]
pub enum Output {
    Exact(Operand),
    Computed(BitIntegerRange, Computation)
}

impl Validate for Output {
    fn validate (&self) -> Result<(), ValidationErrors> {
        ValidationErrors::merge_all(
            Ok(()),
            "output",
            match self {
                Output::Exact(operand) => vec![operand.validate()],
                Output::Computed(range, computation) => vec![range.validate(), computation.validate()],
            }
        )
    }
}

#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize)]
pub struct OperandArg {
    /// # Allocated Operand Bits
    /// Specifies the bits to allocate to this operand argument.
    #[validate]
    pub bits: BitIntegerRange,
    /// # Operand Argument Name
    /// Name of the operand argument. Argument with the name of "inputs" is
    /// reserved so that it wont be be typed inside <> and its value needed to
    /// construct the operand will be the number of items inside the opcode's
    /// parens (computation will apply to this value if provided).
    #[validate]
    pub name: RainSymbol,
    /// # Operand Argument Description
    /// Description of the operand argument.
    #[serde(default)]
    #[validate]
    pub desc: Description,
    /// # Allocated Operand Bits Computation
    /// Specifies any arithmetical operation that needs to be applied to the
    /// value of this operand argument. It will apply to the value before it be
    /// validated by the provided range. The "arg" keyword is reserved for
    /// accessing the value of this operand argument, example: "(arg + 1) * 2".
    #[serde(default)]
    #[validate]
    pub computation: Option<Computation>,
    /// # Operand Argument Range
    /// Determines the valid range of the operand argument after computation
    /// applied. For example an operand argument can be any value between range
    /// of 1 - 10: [[1, 10]] or an operand argument can only be certain exact
    /// values: [[2], [3], [9]], meaning it can only be 2 or 3 or 9.
    #[serde(default)]
    #[validate]
    pub valid_range: Option<Vec<OperandArgRange>>,
}

