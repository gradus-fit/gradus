// SPDX-License-Identifier: AGPL-3.0-or-later

//! OpenWeight JSON Schema validation, parsing, and serialization for Gradus.

mod api;
mod private;

pub use api::{
    Bodyweight, BodyweightEntry, DistanceUnit, DocumentKind, DurationPr, E1RmFormula, Estimated1Rm,
    Exercise, ExerciseLog, ExerciseRecord, ExerciseTemplate, Extensions, Height, HeightUnit,
    Integer, LiftScores, LifterProfile, NormalizedScores, ParseError, ProfileExercise, Program,
    ProgramWeek, RepMax, RepMaxType, SerializationError, SetLog, SetTemplate, Sex, ValidationError,
    ValidationResult, VolumePr, WeightUnit, WorkoutLog, WorkoutTemplate, parse_lifter_profile,
    parse_program, parse_workout_log, parse_workout_template, serialize_lifter_profile,
    serialize_lifter_profile_pretty, serialize_program, serialize_program_pretty,
    serialize_workout_log, serialize_workout_log_pretty, serialize_workout_template,
    serialize_workout_template_pretty, validate, validate_lifter_profile, validate_program,
    validate_workout_log, validate_workout_template,
};
