// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::OnceLock;

use jsonschema::{Resource, Validator};
use serde_json::Value;

use crate::api::{DocumentKind, ValidationError, ValidationResult};

const WORKOUT_LOG_SCHEMA: &str = include_str!("../../schemas/workout-log.schema.json");
const WORKOUT_TEMPLATE_SCHEMA: &str = include_str!("../../schemas/workout-template.schema.json");
const PROGRAM_SCHEMA: &str = include_str!("../../schemas/program.schema.json");
const LIFTER_PROFILE_SCHEMA: &str = include_str!("../../schemas/lifter-profile.schema.json");
const WORKOUT_TEMPLATE_SCHEMA_ID: &str =
    "https://openweight.org/schemas/workout-template.schema.json";

struct Validators {
    workout_log: Validator,
    workout_template: Validator,
    program: Validator,
    lifter_profile: Validator,
}

impl Validators {
    fn build() -> Result<Self, String> {
        let workout_log = schema(WORKOUT_LOG_SCHEMA)?;
        let workout_template = schema(WORKOUT_TEMPLATE_SCHEMA)?;
        let program = schema(PROGRAM_SCHEMA)?;
        let lifter_profile = schema(LIFTER_PROFILE_SCHEMA)?;

        Ok(Self {
            workout_log: validator(&workout_log)?,
            workout_template: validator(&workout_template)?,
            program: jsonschema::draft7::options()
                .should_validate_formats(true)
                .with_resource(
                    WORKOUT_TEMPLATE_SCHEMA_ID,
                    Resource::from_contents(workout_template),
                )
                .build(&program)
                .map_err(|error| error.to_string())?,
            lifter_profile: validator(&lifter_profile)?,
        })
    }

    fn for_kind(&self, kind: DocumentKind) -> &Validator {
        match kind {
            DocumentKind::WorkoutLog => &self.workout_log,
            DocumentKind::WorkoutTemplate => &self.workout_template,
            DocumentKind::Program => &self.program,
            DocumentKind::LifterProfile => &self.lifter_profile,
        }
    }
}

static VALIDATORS: OnceLock<Validators> = OnceLock::new();

pub(crate) fn validate_document(kind: DocumentKind, document: &Value) -> ValidationResult {
    let errors = validators()
        .for_kind(kind)
        .iter_errors(document)
        .map(|error| ValidationError {
            path: error.instance_path().to_string(),
            message: error.to_string(),
        })
        .collect::<Vec<_>>();

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

fn validators() -> &'static Validators {
    VALIDATORS
        .get_or_init(|| Validators::build().expect("the checked-in OpenWeight schemas compile"))
}

fn schema(source: &str) -> Result<Value, String> {
    let schema = serde_json::from_str(source).map_err(|error| error.to_string())?;

    if jsonschema::draft7::meta::is_valid(&schema) {
        Ok(schema)
    } else {
        Err("the schema is not valid JSON Schema Draft-07".to_owned())
    }
}

fn validator(schema: &Value) -> Result<Validator, String> {
    jsonschema::draft7::options()
        .should_validate_formats(true)
        .build(schema)
        .map_err(|error| error.to_string())
}
