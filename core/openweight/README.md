# `gradus-openweight`

Typed Rust parsing, validation, and serialization for OpenWeight `v0.13.0` JSON documents.

The module supports the four OpenWeight root document kinds:

- [`WorkoutLog`](#workout-log-models) — a completed workout;
- [`WorkoutTemplate`](#workout-template-models) — a planned workout;
- [`Program`](#program-models) — a multi-week collection of templates; and
- [`LifterProfile`](#lifter-profile-models) — body measurements and personal records.

## Compatibility

- The caller **must choose the root document kind**. OpenWeight has no unambiguous discriminator, so this module never auto-detects a file type.
- Parsing accepts UTF-8 JSON text, validates it against the selected Draft-07 schema, then returns typed data.
- Serialization converts typed data to JSON, validates the generated JSON against the selected schema, and only then returns it.
- Application-defined fields are retained in [`Extensions`](#extensions) at every object level.
- Typed parse/serialize round trips preserve OpenWeight values and extension fields.

OpenWeight `v0.13.0` is alpha and may introduce breaking changes before 1.0.0. This API accepts the documented `v0.13.0` document shapes.

## Quick start

```rust
use gradus_openweight::{parse_workout_log, serialize_workout_log};

fn copy_workout(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let workout = parse_workout_log(input)?;
    serialize_workout_log(&workout).map_err(Into::into)
}
```

Use the `*_pretty` serializer when human-readable output is needed:

```rust
use gradus_openweight::{parse_workout_template, serialize_workout_template_pretty};

fn format_template(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let template = parse_workout_template(input)?;
    serialize_workout_template_pretty(&template).map_err(Into::into)
}
```

## Root-document API

### Validation

Validation operates on a `serde_json::Value` and returns every error reported by the schema validator.

```rust
pub fn validate(kind: DocumentKind, document: &serde_json::Value) -> ValidationResult;
pub fn validate_workout_log(document: &serde_json::Value) -> ValidationResult;
pub fn validate_workout_template(document: &serde_json::Value) -> ValidationResult;
pub fn validate_program(document: &serde_json::Value) -> ValidationResult;
pub fn validate_lifter_profile(document: &serde_json::Value) -> ValidationResult;
```

`DocumentKind` has the variants `WorkoutLog`, `WorkoutTemplate`, `Program`, and `LifterProfile`.

```rust
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

pub struct ValidationError {
    pub path: String,
    pub message: String,
}
```

`path` is the validator's JSON-instance location. It may be empty for a root-level error.

### Parsing

| Function | Return type |
|---|---|
| `parse_workout_log(&str)` | `Result<WorkoutLog, ParseError>` |
| `parse_workout_template(&str)` | `Result<WorkoutTemplate, ParseError>` |
| `parse_program(&str)` | `Result<Program, ParseError>` |
| `parse_lifter_profile(&str)` | `Result<LifterProfile, ParseError>` |

Each parser first decodes JSON, then validates it, then decodes it into the documented model.

### Serialization

| Compact JSON | Pretty JSON | Input |
|---|---|---|
| `serialize_workout_log` | `serialize_workout_log_pretty` | `&WorkoutLog` |
| `serialize_workout_template` | `serialize_workout_template_pretty` | `&WorkoutTemplate` |
| `serialize_program` | `serialize_program_pretty` | `&Program` |
| `serialize_lifter_profile` | `serialize_lifter_profile_pretty` | `&LifterProfile` |

All serializers return `Result<String, SerializationError>`. Compact output has no insignificant whitespace; pretty output uses `serde_json` indentation.

## Errors

```rust
pub enum ParseError {
    InvalidJson { message: String },
    SchemaValidation { errors: Vec<ValidationError> },
    ModelDeserialization { message: String },
}

pub enum SerializationError {
    JsonEncoding { message: String },
    SchemaValidation { errors: Vec<ValidationError> },
}
```

Both error types implement `Display` and `std::error::Error`.

## Common types

### `Extensions`

```rust
pub type Extensions = std::collections::BTreeMap<String, serde_json::Value>;
```

Every model has a public `extensions: Extensions` field. Unknown JSON properties are flattened into this map when parsing and emitted again when serializing.

Use namespaced keys such as `"gradus:sourceId"`. Do not insert a standard OpenWeight property name into `extensions`; it conflicts with the model's typed field and produces ambiguous output.

### `Integer`

```rust
pub type Integer = serde_json::Number;
```

OpenWeight's JSON Schema `integer` keyword accepts JSON numeric forms such as `5`, `5.0`, and `5e0`. `Integer` retains a JSON number rather than narrowing it to `u64`, so all JSON numbers accepted by the selected validator can be represented by the model. Construct ordinary values with `5.into()`.

### Enums

| Rust type | JSON values |
|---|---|
| `WeightUnit` | `"kg"`, `"lb"` |
| `DistanceUnit` | `"m"`, `"km"`, `"ft"`, `"mi"`, `"yd"` |
| `HeightUnit` | `"cm"`, `"in"` |
| `Sex` | `"male"`, `"female"` |
| `E1RmFormula` | `"brzycki"`, `"epley"`, `"lombardi"`, `"mayhew"`, `"oconner"`, `"wathan"` |
| `RepMaxType` | `"actual"`, `"estimated"` |


## Workout-log models

`WorkoutLog` represents actual completed work.

| Model | Required Rust fields | Optional Rust fields |
|---|---|---|
| `WorkoutLog` | `date: String`, `exercises: Vec<ExerciseLog>` | `name`, `notes`, `duration_seconds`, `template_id` |
| `ExerciseLog` | `exercise: Exercise`, `sets: Vec<SetLog>` | `order`, `notes`, `superset_id` |
| `Exercise` | `name: String` | `equipment`, `category`, `muscles_worked` |
| `SetLog` | none | `reps`, `weight`, `unit`, `duration_seconds`, `distance`, `distance_unit`, `rpe`, `rir`, `to_failure`, `set_type`, `rest_seconds`, `tempo`, `notes`, `target_reps`, `target_weight`, `target_rpe` |

JSON names use OpenWeight camel case. The intentional Rust-to-JSON exceptions are:

| Rust field | JSON property |
|---|---|
| `muscles_worked` | `musclesWorked` |
| `duration_seconds` | `durationSeconds` |
| `distance_unit` | `distanceUnit` |
| `to_failure` | `toFailure` |
| `set_type` | `type` |
| `rest_seconds` | `restSeconds` |
| `target_reps` | `targetReps` |
| `target_weight` | `targetWeight` |
| `target_rpe` | `targetRPE` |
| `superset_id` | `supersetId` |
| `template_id` | `templateId` |

Schema rules include: `weight` and `targetWeight` require `unit`; `distance` requires `distanceUnit`; `exercises` and each exercise's `sets` must be non-empty. A `SetLog` otherwise has no required standard property, so an empty set object is valid.

## Workout-template models

`WorkoutTemplate` represents planned work.

| Model | Required Rust fields | Optional Rust fields |
|---|---|---|
| `WorkoutTemplate` | `name: String`, `exercises: Vec<ExerciseTemplate>` | `notes`, `day` |
| `ExerciseTemplate` | `exercise: Exercise`, `sets: Vec<SetTemplate>` | `order`, `notes`, `superset_id` |
| `SetTemplate` | none | `target_reps`, `target_reps_min`, `target_reps_max`, `target_weight`, `unit`, `percentage`, `percentage_of`, `target_rpe`, `target_rir`, `rest_seconds`, `tempo`, `set_type`, `notes` |

`ExerciseTemplate` uses the same `Exercise` type as `ExerciseLog`.

| Rust field | JSON property |
|---|---|
| `target_reps_min` | `targetRepsMin` |
| `target_reps_max` | `targetRepsMax` |
| `target_weight` | `targetWeight` |
| `percentage_of` | `percentageOf` |
| `target_rpe` | `targetRPE` |
| `target_rir` | `targetRIR` |
| `rest_seconds` | `restSeconds` |
| `set_type` | `type` |

Schema rules include: `targetWeight` requires `unit`; `percentage` requires `percentageOf`; `day` is 1 through 7; exercises and their sets must be non-empty.

## Program models

| Model | Required Rust fields | Optional Rust fields |
|---|---|---|
| `Program` | `name: String`, `weeks: Vec<ProgramWeek>` | `description`, `author`, `tags` |
| `ProgramWeek` | `workouts: Vec<WorkoutTemplate>` | `name`, `notes` |

`ProgramWeek.workouts` uses the complete `WorkoutTemplate` model.

## Lifter-profile models

| Model | Required Rust fields | Optional Rust fields |
|---|---|---|
| `LifterProfile` | `exported_at: String` | `name`, `sex`, `birth_date`, `height`, `bodyweight`, `bodyweight_history`, `records`, `normalized_scores` |
| `Height` | `value: f64`, `unit: HeightUnit` | none |
| `Bodyweight` | `value: f64`, `unit: WeightUnit` | `date` |
| `BodyweightEntry` | `value: f64`, `unit: WeightUnit`, `date: String` | `notes` |
| `ExerciseRecord` | `exercise: ProfileExercise` | `rep_maxes`, `estimated_1rm`, `volume_pr`, `duration_pr` |
| `ProfileExercise` | `name: String` | `equipment`, `category` |
| `RepMax` | `reps: Integer`, `weight: f64`, `unit: WeightUnit`, `date: String` | `rep_max_type`, `bodyweight_kg`, `workout_id`, `rpe`, `notes` |
| `Estimated1Rm` | `value: f64`, `unit: WeightUnit`, `formula: E1RmFormula`, `based_on_reps: Integer`, `based_on_weight: f64` | `date` |
| `VolumePr` | `value: f64`, `unit: WeightUnit`, `date: String` | `notes` |
| `DurationPr` | `seconds: Integer`, `date: String` | `weight`, `unit`, `notes` |
| `NormalizedScores` | none | `squat`, `bench`, `deadlift`, `total` |
| `LiftScores` | none | `wilks`, `dots`, `ipf_gl`, `glossbrenner` |

Important JSON-name mappings:

| Rust field | JSON property |
|---|---|
| `exported_at` | `exportedAt` |
| `birth_date` | `birthDate` |
| `bodyweight_history` | `bodyweightHistory` |
| `rep_maxes` | `repMaxes` |
| `estimated_1rm` | `estimated1RM` |
| `volume_pr` | `volumePR` |
| `duration_pr` | `durationPR` |
| `rep_max_type` | `type` |
| `bodyweight_kg` | `bodyweightKg` |
| `workout_id` | `workoutId` |
| `based_on_reps` | `basedOnReps` |
| `based_on_weight` | `basedOnWeight` |
| `ipf_gl` | `ipfGl` |

`ProfileExercise` is deliberately distinct from `Exercise`: the LifterProfile schema does not standardize `musclesWorked`. Therefore `musclesWorked`, including values that are not a string array, is retained as an extension on `ProfileExercise` instead of being rejected or coerced.

## Dates, numbers, and format behavior

Dates and date-times are `String` fields, so parsing does not normalize timezone or calendar text. Validation enforces their documented formats.

Date-times require a `T` separator; for example, `"2024-01-15 09:00:00Z"` is rejected.

Standard measurement and score values are `f64`. Non-finite values cannot be serialized as JSON and serialization returns an error. Schema limits such as non-negative values, RPE ranges, enums, string lengths, tempo patterns, and conditional unit requirements are enforced by schema validation rather than by constructors.

## Untrusted input

Callers handling untrusted JSON should enforce input-size limits, request deadlines, and a cap on returned validation errors before invoking a parser. Extension values are untrusted application data and require separate validation before being used for authorization, URLs, paths, or queries.
