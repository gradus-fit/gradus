// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::private::{parse_document, serialize_document, validate_document};

/// Application-defined OpenWeight fields preserved during parsing and serialization.
pub type Extensions = BTreeMap<String, Value>;

/// A JSON number that the selected schema has validated as an integer.
pub type Integer = serde_json::Number;

/// A root OpenWeight document schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    WorkoutLog,
    WorkoutTemplate,
    Program,
    LifterProfile,
}

/// One JSON Schema validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

/// The result of validating an OpenWeight JSON value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

/// An error returned when parsing an OpenWeight JSON document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidJson { message: String },
    SchemaValidation { errors: Vec<ValidationError> },
    ModelDeserialization { message: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { message } => write!(formatter, "invalid JSON: {message}"),
            Self::SchemaValidation { .. } => formatter.write_str("schema validation failed"),
            Self::ModelDeserialization { message } => {
                write!(formatter, "validated JSON could not be decoded: {message}")
            }
        }
    }
}

impl Error for ParseError {}

/// An error returned when serializing an OpenWeight document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    JsonEncoding { message: String },
    SchemaValidation { errors: Vec<ValidationError> },
}

impl fmt::Display for SerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonEncoding { message } => write!(formatter, "JSON encoding failed: {message}"),
            Self::SchemaValidation { .. } => formatter.write_str("schema validation failed"),
        }
    }
}

impl Error for SerializationError {}

/// A unit for a weight measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeightUnit {
    Kg,
    Lb,
}

/// A unit for a distance measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistanceUnit {
    M,
    Km,
    Ft,
    Mi,
    Yd,
}

/// A unit for a height measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeightUnit {
    Cm,
    In,
}

/// Biological sex used by OpenWeight score calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// A formula used to calculate an estimated one-repetition maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum E1RmFormula {
    Brzycki,
    Epley,
    Lombardi,
    Mayhew,
    Oconner,
    Wathan,
}

/// Whether a rep maximum was actually tested or estimated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepMaxType {
    Actual,
    Estimated,
}

/// An exercise used in a workout or workout template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muscles_worked: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// An exercise used in a lifter profile record.
///
/// OpenWeight's lifter-profile schema does not standardize `musclesWorked`, so
/// that key is preserved in `extensions` even when it is present.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileExercise {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A completed set in a workout log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<WeightUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_unit: Option<DistanceUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rir: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_failure: Option<bool>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub set_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_seconds: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_reps: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_weight: Option<f64>,
    #[serde(rename = "targetRPE", skip_serializing_if = "Option::is_none")]
    pub target_rpe: Option<f64>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A performed exercise and its completed sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseLog {
    pub exercise: Exercise,
    pub sets: Vec<SetLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superset_id: Option<Integer>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A completed strength-training session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkoutLog {
    pub date: String,
    pub exercises: Vec<ExerciseLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A prescribed set in a workout template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_reps: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_reps_min: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_reps_max: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<WeightUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of: Option<String>,
    #[serde(rename = "targetRPE", skip_serializing_if = "Option::is_none")]
    pub target_rpe: Option<f64>,
    #[serde(rename = "targetRIR", skip_serializing_if = "Option::is_none")]
    pub target_rir: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_seconds: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub set_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A planned exercise and its prescribed sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseTemplate {
    pub exercise: Exercise,
    pub sets: Vec<SetTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Integer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superset_id: Option<Integer>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A planned workout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkoutTemplate {
    pub name: String,
    pub exercises: Vec<ExerciseTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<Integer>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A week in a training program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramWeek {
    pub workouts: Vec<WorkoutTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A multi-week training program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub name: String,
    pub weeks: Vec<ProgramWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A height measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Height {
    pub value: f64,
    pub unit: HeightUnit,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A bodyweight measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bodyweight {
    pub value: f64,
    pub unit: WeightUnit,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A historical bodyweight measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyweightEntry {
    pub value: f64,
    pub unit: WeightUnit,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A repetition maximum for an exercise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepMax {
    pub reps: Integer,
    pub weight: f64,
    pub unit: WeightUnit,
    pub date: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub rep_max_type: Option<RepMaxType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bodyweight_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workout_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// An estimated one-repetition maximum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimated1Rm {
    pub value: f64,
    pub unit: WeightUnit,
    pub formula: E1RmFormula,
    pub based_on_reps: Integer,
    pub based_on_weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A volume personal record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumePr {
    pub value: f64,
    pub unit: WeightUnit,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A duration personal record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurationPr {
    pub seconds: Integer,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<WeightUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// Personal records for an exercise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseRecord {
    pub exercise: ProfileExercise,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rep_maxes: Option<Vec<RepMax>>,
    #[serde(rename = "estimated1RM", skip_serializing_if = "Option::is_none")]
    pub estimated_1rm: Option<Estimated1Rm>,
    #[serde(rename = "volumePR", skip_serializing_if = "Option::is_none")]
    pub volume_pr: Option<VolumePr>,
    #[serde(rename = "durationPR", skip_serializing_if = "Option::is_none")]
    pub duration_pr: Option<DurationPr>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// Bodyweight-normalized scores for one lift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiftScores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wilks: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dots: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipf_gl: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossbrenner: Option<f64>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// Bodyweight-normalized scores across powerlifts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedScores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squat: Option<LiftScores>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bench: Option<LiftScores>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadlift: Option<LiftScores>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<LiftScores>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// A lifter profile containing measurements and personal records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifterProfile {
    pub exported_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bodyweight: Option<Bodyweight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bodyweight_history: Option<Vec<BodyweightEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub records: Option<Vec<ExerciseRecord>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_scores: Option<NormalizedScores>,
    #[serde(default, flatten)]
    pub extensions: Extensions,
}

/// Validates `document` against the schema selected by `kind`.
///
/// The caller must declare the document kind because OpenWeight files do not
/// contain an unambiguous root discriminator.
pub fn validate(kind: DocumentKind, document: &Value) -> ValidationResult {
    validate_document(kind, document)
}

/// Validates a workout-log JSON value.
pub fn validate_workout_log(document: &Value) -> ValidationResult {
    validate(DocumentKind::WorkoutLog, document)
}

/// Validates a workout-template JSON value.
pub fn validate_workout_template(document: &Value) -> ValidationResult {
    validate(DocumentKind::WorkoutTemplate, document)
}

/// Validates a program JSON value.
pub fn validate_program(document: &Value) -> ValidationResult {
    validate(DocumentKind::Program, document)
}

/// Validates a lifter-profile JSON value.
pub fn validate_lifter_profile(document: &Value) -> ValidationResult {
    validate(DocumentKind::LifterProfile, document)
}

/// Parses and validates a workout-log JSON document.
pub fn parse_workout_log(json: &str) -> Result<WorkoutLog, ParseError> {
    parse_document(DocumentKind::WorkoutLog, json)
}

/// Parses and validates a workout-template JSON document.
pub fn parse_workout_template(json: &str) -> Result<WorkoutTemplate, ParseError> {
    parse_document(DocumentKind::WorkoutTemplate, json)
}

/// Parses and validates a program JSON document.
pub fn parse_program(json: &str) -> Result<Program, ParseError> {
    parse_document(DocumentKind::Program, json)
}

/// Parses and validates a lifter-profile JSON document.
pub fn parse_lifter_profile(json: &str) -> Result<LifterProfile, ParseError> {
    parse_document(DocumentKind::LifterProfile, json)
}

/// Serializes a workout log as compact, schema-valid OpenWeight JSON.
pub fn serialize_workout_log(workout: &WorkoutLog) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::WorkoutLog, workout, false)
}

/// Serializes a workout log as pretty-printed, schema-valid OpenWeight JSON.
pub fn serialize_workout_log_pretty(workout: &WorkoutLog) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::WorkoutLog, workout, true)
}

/// Serializes a workout template as compact, schema-valid OpenWeight JSON.
pub fn serialize_workout_template(
    template: &WorkoutTemplate,
) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::WorkoutTemplate, template, false)
}

/// Serializes a workout template as pretty-printed, schema-valid OpenWeight JSON.
pub fn serialize_workout_template_pretty(
    template: &WorkoutTemplate,
) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::WorkoutTemplate, template, true)
}

/// Serializes a program as compact, schema-valid OpenWeight JSON.
pub fn serialize_program(program: &Program) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::Program, program, false)
}

/// Serializes a program as pretty-printed, schema-valid OpenWeight JSON.
pub fn serialize_program_pretty(program: &Program) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::Program, program, true)
}

/// Serializes a lifter profile as compact, schema-valid OpenWeight JSON.
pub fn serialize_lifter_profile(profile: &LifterProfile) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::LifterProfile, profile, false)
}

/// Serializes a lifter profile as pretty-printed, schema-valid OpenWeight JSON.
pub fn serialize_lifter_profile_pretty(
    profile: &LifterProfile,
) -> Result<String, SerializationError> {
    serialize_document(DocumentKind::LifterProfile, profile, true)
}
