// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{error::Error, fs, path::Path};

use gradus_openweight::{
    DocumentKind, Exercise, ExerciseLog, ExerciseTemplate, Extensions, LifterProfile, ParseError,
    Program, ProgramWeek, SerializationError, SetLog, SetTemplate, ValidationError, WorkoutLog,
    WorkoutTemplate, parse_lifter_profile, parse_program, parse_workout_log,
    parse_workout_template, serialize_lifter_profile, serialize_lifter_profile_pretty,
    serialize_program, serialize_program_pretty, serialize_workout_log,
    serialize_workout_log_pretty, serialize_workout_template, serialize_workout_template_pretty,
    validate_lifter_profile, validate_program, validate_workout_log, validate_workout_template,
};
use serde_json::{Value, json};

#[test]
fn every_official_valid_fixture_parses_and_serializes_semantically() {
    for (directory, kind) in [
        ("workout-logs", DocumentKind::WorkoutLog),
        ("workout-templates", DocumentKind::WorkoutTemplate),
        ("programs", DocumentKind::Program),
        ("lifter-profiles", DocumentKind::LifterProfile),
    ] {
        for path in fixture_paths(directory) {
            let source = fs::read_to_string(&path).expect("fixture is readable");
            match kind {
                DocumentKind::WorkoutLog => {
                    let parsed =
                        fixture_result(&path, "does not parse", parse_workout_log(&source));
                    let serialized =
                        fixture_result(&path, "does not serialize", serialize_workout_log(&parsed));
                    let round_tripped = fixture_result(
                        &path,
                        "serialized output does not parse",
                        parse_workout_log(&serialized),
                    );
                    assert_eq!(round_tripped, parsed, "{}", path.display());
                }
                DocumentKind::WorkoutTemplate => {
                    let parsed =
                        fixture_result(&path, "does not parse", parse_workout_template(&source));
                    let serialized = fixture_result(
                        &path,
                        "does not serialize",
                        serialize_workout_template(&parsed),
                    );
                    let round_tripped = fixture_result(
                        &path,
                        "serialized output does not parse",
                        parse_workout_template(&serialized),
                    );
                    assert_eq!(round_tripped, parsed, "{}", path.display());
                }
                DocumentKind::Program => {
                    let parsed = fixture_result(&path, "does not parse", parse_program(&source));
                    let serialized =
                        fixture_result(&path, "does not serialize", serialize_program(&parsed));
                    let round_tripped = fixture_result(
                        &path,
                        "serialized output does not parse",
                        parse_program(&serialized),
                    );
                    assert_eq!(round_tripped, parsed, "{}", path.display());
                }
                DocumentKind::LifterProfile => {
                    let parsed =
                        fixture_result(&path, "does not parse", parse_lifter_profile(&source));
                    let serialized = fixture_result(
                        &path,
                        "does not serialize",
                        serialize_lifter_profile(&parsed),
                    );
                    let round_tripped = fixture_result(
                        &path,
                        "serialized output does not parse",
                        parse_lifter_profile(&serialized),
                    );
                    assert_eq!(round_tripped, parsed, "{}", path.display());
                }
            }
        }
    }
}

#[test]
fn type_specific_validation_and_pretty_serialization_work_for_every_document_kind() {
    let workout = fixture("workout-logs", "full-featured.json");
    let template = fixture("workout-templates", "full-featured.json");
    let program = fixture("programs", "531-bbb.json");
    let profile = fixture("lifter-profiles", "full-featured.json");

    assert!(validate_workout_log(&workout).valid);
    assert!(validate_workout_template(&template).valid);
    assert!(validate_program(&program).valid);
    assert!(validate_lifter_profile(&profile).valid);

    assert!(
        serialize_workout_log_pretty(&parse_workout_log_value(&workout))
            .unwrap()
            .contains('\n')
    );
    assert!(
        serialize_workout_template_pretty(&parse_workout_template_value(&template))
            .unwrap()
            .contains('\n')
    );
    assert!(
        serialize_program_pretty(&parse_program_value(&program))
            .unwrap()
            .contains('\n')
    );
    assert!(
        serialize_lifter_profile_pretty(&parse_lifter_profile_value(&profile))
            .unwrap()
            .contains('\n')
    );
}

#[test]
fn maps_acronym_fields_and_json_schema_integer_representations() {
    let template = json!({
        "name": "Template",
        "exercises": [{
            "exercise": { "name": "Squat" },
            "sets": [{ "targetReps": 5e0, "targetRPE": 8.5, "targetRIR": 2.0 }]
        }]
    });
    let workout = json!({
        "date": "2024-01-15T09:00:00Z",
        "exercises": [{ "exercise": { "name": "Squat" }, "sets": [{ "reps": 5.0 }] }]
    });

    let parsed_template = parse_workout_template_value(&template);
    let set = &parsed_template.exercises[0].sets[0];
    assert_eq!(set.target_rpe, Some(8.5));
    assert_eq!(
        set.target_rir.as_ref().and_then(serde_json::Number::as_f64),
        Some(2.0)
    );
    assert_eq!(
        set.target_reps
            .as_ref()
            .and_then(serde_json::Number::as_f64),
        Some(5.0)
    );
    let serialized_template: Value = serde_json::from_str(
        &serialize_workout_template(&parsed_template).expect("template serializes"),
    )
    .expect("serialized template is JSON");
    let serialized_set = &serialized_template["exercises"][0]["sets"][0];
    assert_eq!(serialized_set["targetRPE"], json!(8.5));
    assert_eq!(serialized_set["targetRIR"], json!(2.0));
    assert!(serialized_set.get("targetRpe").is_none());
    assert!(serialized_set.get("targetRir").is_none());

    let parsed_workout = parse_workout_log_value(&workout);
    assert_eq!(
        parsed_workout.exercises[0].sets[0]
            .reps
            .as_ref()
            .and_then(serde_json::Number::as_f64),
        Some(5.0)
    );
}

#[test]
fn preserves_schema_specific_lifter_exercise_extensions() {
    let profile = json!({
        "exportedAt": "2024-01-15T09:00:00Z",
        "records": [{ "exercise": { "name": "Squat", "musclesWorked": 42 } }]
    });

    let parsed = parse_lifter_profile_value(&profile);

    assert_eq!(
        parsed.records.as_ref().unwrap()[0]
            .exercise
            .extensions
            .get("musclesWorked"),
        Some(&json!(42))
    );
    let round_tripped =
        parse_lifter_profile(&serialize_lifter_profile(&parsed).expect("profile serializes"))
            .expect("serialized profile parses");
    assert_eq!(round_tripped, parsed);
}

#[test]
fn preserves_extensions_at_every_openweight_object_level() {
    let workout = json!({
        "date": "2024-01-15T09:00:00Z",
        "app:workout": true,
        "exercises": [{
            "app:exerciseLog": true,
            "exercise": { "name": "Squat", "app:exercise": true },
            "sets": [{ "reps": 5, "app:setLog": true }]
        }]
    });
    let template = json!({
        "name": "Template",
        "app:template": true,
        "exercises": [{
            "app:exerciseTemplate": true,
            "exercise": { "name": "Squat", "app:exercise": true },
            "sets": [{ "targetReps": 5, "app:setTemplate": true }]
        }]
    });
    let program = json!({
        "name": "Program",
        "app:program": true,
        "weeks": [{
            "app:week": true,
            "workouts": [template]
        }]
    });
    let profile = json!({
        "exportedAt": "2024-01-15T09:00:00Z",
        "app:profile": true,
        "height": { "value": 180, "unit": "cm", "app:height": true },
        "bodyweight": { "value": 80, "unit": "kg", "app:bodyweight": true },
        "bodyweightHistory": [{ "value": 80, "unit": "kg", "date": "2024-01-15", "app:entry": true }],
        "records": [{
            "app:record": true,
            "exercise": { "name": "Squat", "app:exercise": true },
            "repMaxes": [{ "reps": 1, "weight": 180, "unit": "kg", "date": "2024-01-15", "app:repMax": true }],
            "estimated1RM": { "value": 185, "unit": "kg", "formula": "epley", "basedOnReps": 5, "basedOnWeight": 155, "app:estimated": true },
            "volumePR": { "value": 8500, "unit": "kg", "date": "2024-01-15", "app:volume": true },
            "durationPR": { "seconds": 60, "date": "2024-01-15", "app:duration": true }
        }],
        "normalizedScores": { "app:scores": true, "squat": { "wilks": 100, "app:lift": true } }
    });

    assert_round_trip_workout(workout);
    assert_round_trip_template(template);
    assert_round_trip_program(program);
    assert_round_trip_profile(profile);
}

#[test]
fn parse_errors_distinguish_invalid_json_and_schema_failures() {
    assert!(matches!(
        parse_workout_log("not JSON"),
        Err(ParseError::InvalidJson { .. })
    ));
    assert!(matches!(
        parse_workout_log(r#"{"date":"2024-01-15T09:00:00Z"}"#),
        Err(ParseError::SchemaValidation { .. })
    ));
}

#[test]
fn every_parser_reports_document_specific_invalid_json_and_schema_errors() {
    assert_invalid_json(parse_workout_log("not JSON"));
    assert_invalid_json(parse_workout_template("not JSON"));
    assert_invalid_json(parse_program("not JSON"));
    assert_invalid_json(parse_lifter_profile("not JSON"));

    assert_schema_error(
        parse_workout_log(
            r#"{"date":"not-a-date","exercises":[{"exercise":{"name":"Squat"},"sets":[{}]}]}"#,
        ),
        "/date",
    );
    assert_schema_error(
        parse_workout_template(
            r#"{"name":"Template","day":0,"exercises":[{"exercise":{"name":"Squat"},"sets":[{}]}]}"#,
        ),
        "/day",
    );
    assert_schema_error(parse_program(r#"{"name":"","weeks":[]}"#), "/name");
    assert_schema_error(
        parse_lifter_profile(r#"{"exportedAt":"not-a-date"}"#),
        "/exportedAt",
    );
}

#[test]
fn constructed_models_serialize_without_optional_fields() {
    let workout = WorkoutLog {
        date: "2024-01-15T09:00:00Z".to_owned(),
        exercises: vec![ExerciseLog {
            exercise: Exercise {
                name: "Squat".to_owned(),
                equipment: None,
                category: None,
                muscles_worked: None,
                extensions: Extensions::new(),
            },
            sets: vec![SetLog {
                reps: None,
                weight: None,
                unit: None,
                duration_seconds: None,
                distance: None,
                distance_unit: None,
                rpe: None,
                rir: None,
                to_failure: None,
                set_type: None,
                rest_seconds: None,
                tempo: None,
                notes: None,
                target_reps: None,
                target_weight: None,
                target_rpe: None,
                extensions: Extensions::new(),
            }],
            order: None,
            notes: None,
            superset_id: None,
            extensions: Extensions::new(),
        }],
        name: None,
        notes: None,
        duration_seconds: None,
        template_id: None,
        extensions: Extensions::new(),
    };
    let template = WorkoutTemplate {
        name: "Day 1".to_owned(),
        exercises: vec![ExerciseTemplate {
            exercise: Exercise {
                name: "Squat".to_owned(),
                equipment: None,
                category: None,
                muscles_worked: None,
                extensions: Extensions::new(),
            },
            sets: vec![SetTemplate {
                target_reps: None,
                target_reps_min: None,
                target_reps_max: None,
                target_weight: None,
                unit: None,
                percentage: None,
                percentage_of: None,
                target_rpe: None,
                target_rir: None,
                rest_seconds: None,
                tempo: None,
                set_type: None,
                notes: None,
                extensions: Extensions::new(),
            }],
            order: None,
            notes: None,
            superset_id: None,
            extensions: Extensions::new(),
        }],
        notes: None,
        day: None,
        extensions: Extensions::new(),
    };
    let program = Program {
        name: "Program".to_owned(),
        weeks: vec![ProgramWeek {
            workouts: vec![template.clone()],
            name: None,
            notes: None,
            extensions: Extensions::new(),
        }],
        description: None,
        author: None,
        tags: None,
        extensions: Extensions::new(),
    };
    let profile = LifterProfile {
        exported_at: "2024-01-15T09:00:00Z".to_owned(),
        name: None,
        sex: None,
        birth_date: None,
        height: None,
        bodyweight: None,
        bodyweight_history: None,
        records: None,
        normalized_scores: None,
        extensions: Extensions::new(),
    };

    assert_constructed_model_serializes(
        &workout,
        json!({"date":"2024-01-15T09:00:00Z","exercises":[{"exercise":{"name":"Squat"},"sets":[{}]}]}),
        serialize_workout_log,
        serialize_workout_log_pretty,
        parse_workout_log,
    );
    assert_constructed_model_serializes(
        &template,
        json!({"name":"Day 1","exercises":[{"exercise":{"name":"Squat"},"sets":[{}]}]}),
        serialize_workout_template,
        serialize_workout_template_pretty,
        parse_workout_template,
    );
    assert_constructed_model_serializes(
        &program,
        json!({"name":"Program","weeks":[{"workouts":[{"name":"Day 1","exercises":[{"exercise":{"name":"Squat"},"sets":[{}]}]}]}]}),
        serialize_program,
        serialize_program_pretty,
        parse_program,
    );
    assert_constructed_model_serializes(
        &profile,
        json!({"exportedAt":"2024-01-15T09:00:00Z"}),
        serialize_lifter_profile,
        serialize_lifter_profile_pretty,
        parse_lifter_profile,
    );
}

#[test]
fn serialization_rejects_invalid_client_data_and_non_json_numbers() {
    let mut invalid_schema = minimal_workout();
    invalid_schema.date = "not-a-date".to_owned();
    let mut non_json_number = minimal_workout();
    non_json_number.exercises[0].sets[0].weight = Some(f64::NAN);

    assert!(matches!(
        serialize_workout_log(&invalid_schema),
        Err(SerializationError::SchemaValidation { .. })
    ));
    assert!(matches!(
        serialize_workout_log(&non_json_number),
        Err(SerializationError::SchemaValidation { .. })
    ));
}

#[test]
fn public_error_messages_are_actionable() {
    let validation_error = ValidationError {
        path: "/date".to_owned(),
        message: "invalid date".to_owned(),
    };
    let errors = vec![validation_error];
    let parse_errors = [
        ParseError::InvalidJson {
            message: "expected value".to_owned(),
        },
        ParseError::SchemaValidation {
            errors: errors.clone(),
        },
        ParseError::ModelDeserialization {
            message: "unexpected type".to_owned(),
        },
    ];
    let serialization_errors = [
        SerializationError::JsonEncoding {
            message: "non-finite number".to_owned(),
        },
        SerializationError::SchemaValidation { errors },
    ];

    assert_eq!(parse_errors[0].to_string(), "invalid JSON: expected value");
    assert_eq!(parse_errors[1].to_string(), "schema validation failed");
    assert_eq!(
        parse_errors[2].to_string(),
        "validated JSON could not be decoded: unexpected type"
    );
    assert_eq!(
        serialization_errors[0].to_string(),
        "JSON encoding failed: non-finite number"
    );
    assert_eq!(
        serialization_errors[1].to_string(),
        "schema validation failed"
    );
    assert_error(&parse_errors[0]);
    assert_error(&serialization_errors[0]);
}

fn assert_invalid_json<T>(result: Result<T, ParseError>) {
    assert!(matches!(result, Err(ParseError::InvalidJson { .. })));
}

fn assert_schema_error<T>(result: Result<T, ParseError>, expected_path: &str) {
    let Err(ParseError::SchemaValidation { errors }) = result else {
        panic!("expected schema validation error");
    };

    assert!(!errors.is_empty());
    assert!(errors.iter().any(|error| error.path == expected_path));
}

fn assert_constructed_model_serializes<T>(
    model: &T,
    expected: Value,
    serialize: fn(&T) -> Result<String, SerializationError>,
    serialize_pretty: fn(&T) -> Result<String, SerializationError>,
    parse: fn(&str) -> Result<T, ParseError>,
) where
    T: PartialEq + std::fmt::Debug,
{
    let compact = serialize(model).expect("constructed model serializes compactly");
    let pretty = serialize_pretty(model).expect("constructed model serializes prettily");

    assert!(!compact.contains('\n'));
    assert!(pretty.contains("\n  "));
    assert!(pretty.contains('\n'));
    assert_eq!(serde_json::from_str::<Value>(&compact).unwrap(), expected);
    assert_eq!(serde_json::from_str::<Value>(&pretty).unwrap(), expected);
    assert_eq!(&parse(&compact).expect("compact output parses"), model);
    assert_eq!(&parse(&pretty).expect("pretty output parses"), model);
}

fn fixture_result<T, E: std::fmt::Display>(
    path: &Path,
    operation: &str,
    result: Result<T, E>,
) -> T {
    result.unwrap_or_else(|error| panic!("{} {operation}: {error}", path.display()))
}

fn fixture_paths(directory: &str) -> Vec<std::path::PathBuf> {
    let mut paths = fs::read_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(directory),
    )
    .expect("fixture directory exists")
    .map(|entry| entry.expect("fixture directory entry").path())
    .filter(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    })
    .collect::<Vec<_>>();
    paths.sort();

    paths
}

fn fixture(directory: &str, filename: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(directory)
        .join(filename);
    let source = fs::read_to_string(path).expect("fixture is readable");

    serde_json::from_str(&source).expect("fixture is JSON")
}

fn parse_workout_log_value(value: &Value) -> WorkoutLog {
    parse_workout_log(&value.to_string()).expect("workout parses")
}

fn parse_workout_template_value(value: &Value) -> gradus_openweight::WorkoutTemplate {
    parse_workout_template(&value.to_string()).expect("template parses")
}

fn parse_program_value(value: &Value) -> gradus_openweight::Program {
    parse_program(&value.to_string()).expect("program parses")
}

fn parse_lifter_profile_value(value: &Value) -> gradus_openweight::LifterProfile {
    parse_lifter_profile(&value.to_string()).expect("profile parses")
}

fn assert_round_trip_workout(expected: Value) {
    let parsed = parse_workout_log_value(&expected);
    let round_tripped =
        parse_workout_log(&serialize_workout_log(&parsed).expect("workout serializes"))
            .expect("serialized workout parses");

    assert_eq!(round_tripped, parsed);
    assert_eq!(parsed.extensions.get("app:workout"), Some(&json!(true)));
    assert_eq!(
        parsed.exercises[0].extensions.get("app:exerciseLog"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed.exercises[0].exercise.extensions.get("app:exercise"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed.exercises[0].sets[0].extensions.get("app:setLog"),
        Some(&json!(true))
    );
}

fn assert_round_trip_template(expected: Value) {
    let parsed = parse_workout_template_value(&expected);
    let round_tripped =
        parse_workout_template(&serialize_workout_template(&parsed).expect("template serializes"))
            .expect("serialized template parses");

    assert_eq!(round_tripped, parsed);
    assert_eq!(parsed.extensions.get("app:template"), Some(&json!(true)));
    assert_eq!(
        parsed.exercises[0].extensions.get("app:exerciseTemplate"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed.exercises[0].sets[0]
            .extensions
            .get("app:setTemplate"),
        Some(&json!(true))
    );
}

fn assert_round_trip_program(expected: Value) {
    let parsed = parse_program_value(&expected);
    let round_tripped = parse_program(&serialize_program(&parsed).expect("program serializes"))
        .expect("serialized program parses");

    assert_eq!(round_tripped, parsed);
    assert_eq!(parsed.extensions.get("app:program"), Some(&json!(true)));
    assert_eq!(
        parsed.weeks[0].extensions.get("app:week"),
        Some(&json!(true))
    );
}

fn assert_round_trip_profile(expected: Value) {
    let parsed = parse_lifter_profile_value(&expected);
    let round_tripped =
        parse_lifter_profile(&serialize_lifter_profile(&parsed).expect("profile serializes"))
            .expect("serialized profile parses");

    assert_eq!(round_tripped, parsed);
    assert_eq!(parsed.extensions.get("app:profile"), Some(&json!(true)));
    assert_eq!(
        parsed.height.as_ref().unwrap().extensions.get("app:height"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed
            .bodyweight
            .as_ref()
            .unwrap()
            .extensions
            .get("app:bodyweight"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed.bodyweight_history.as_ref().unwrap()[0]
            .extensions
            .get("app:entry"),
        Some(&json!(true))
    );
    let record = &parsed.records.as_ref().unwrap()[0];
    assert_eq!(record.extensions.get("app:record"), Some(&json!(true)));
    assert_eq!(
        record.rep_maxes.as_ref().unwrap()[0]
            .extensions
            .get("app:repMax"),
        Some(&json!(true))
    );
    assert_eq!(
        record
            .estimated_1rm
            .as_ref()
            .unwrap()
            .extensions
            .get("app:estimated"),
        Some(&json!(true))
    );
    assert_eq!(
        record
            .volume_pr
            .as_ref()
            .unwrap()
            .extensions
            .get("app:volume"),
        Some(&json!(true))
    );
    assert_eq!(
        record
            .duration_pr
            .as_ref()
            .unwrap()
            .extensions
            .get("app:duration"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed
            .normalized_scores
            .as_ref()
            .unwrap()
            .extensions
            .get("app:scores"),
        Some(&json!(true))
    );
    assert_eq!(
        parsed
            .normalized_scores
            .as_ref()
            .unwrap()
            .squat
            .as_ref()
            .unwrap()
            .extensions
            .get("app:lift"),
        Some(&json!(true))
    );
}

fn minimal_workout() -> WorkoutLog {
    WorkoutLog {
        date: "2024-01-15T09:00:00Z".to_owned(),
        exercises: vec![ExerciseLog {
            exercise: Exercise {
                name: "Squat".to_owned(),
                equipment: None,
                category: None,
                muscles_worked: None,
                extensions: Extensions::new(),
            },
            sets: vec![SetLog {
                reps: Some(5.into()),
                weight: None,
                unit: None,
                duration_seconds: None,
                distance: None,
                distance_unit: None,
                rpe: None,
                rir: None,
                to_failure: None,
                set_type: None,
                rest_seconds: None,
                tempo: None,
                notes: None,
                target_reps: None,
                target_weight: None,
                target_rpe: None,
                extensions: Extensions::new(),
            }],
            order: None,
            notes: None,
            superset_id: None,
            extensions: Extensions::new(),
        }],
        name: None,
        notes: None,
        duration_seconds: None,
        template_id: None,
        extensions: Extensions::new(),
    }
}

fn assert_error(error: &dyn Error) {
    assert!(!error.to_string().is_empty());
}
