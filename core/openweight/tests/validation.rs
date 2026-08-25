// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::BTreeSet, fs, path::Path};

use gradus_openweight::{DocumentKind, validate};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[test]
fn accepts_every_official_valid_fixture() {
    for (directory, kind) in [
        ("workout-logs", DocumentKind::WorkoutLog),
        ("workout-templates", DocumentKind::WorkoutTemplate),
        ("programs", DocumentKind::Program),
        ("lifter-profiles", DocumentKind::LifterProfile),
    ] {
        for (path, fixture) in fixtures_in(directory) {
            let result = validate(kind, &fixture);

            assert!(
                result.valid,
                "{} failed: {:?}",
                path.display(),
                result.errors
            );
            assert!(result.errors.is_empty());
        }
    }
}

#[test]
fn rejects_every_official_invalid_fixture() {
    for (path, fixture) in fixtures_in("invalid") {
        let result = validate(kind_for_invalid_fixture(&path), &fixture);

        assert!(!result.valid, "invalid fixture passed: {}", path.display());
        assert!(!result.errors.is_empty());
    }
}

#[test]
fn baseline_matches_the_pinned_schema_and_fixture_manifest() {
    assert_eq!(fixture_paths(), expected_fixture_paths());

    for (filename, expected_hash) in [
        (
            "lifter-profile.schema.json",
            "ee234da1628984ac4c4db2da278bc18c6bca4470267f9db34334e8fd561c3c42",
        ),
        (
            "program.schema.json",
            "c40fd4a0e82293a11a58320be6f37297bc2c1d9932bb07e077d7ebc16d24a848",
        ),
        (
            "workout-log.schema.json",
            "6c1402a9f47ea60d28bd2583f9eae66ce93b35c89da1641eecbec6c4711f9007",
        ),
        (
            "workout-template.schema.json",
            "849080672d86d2314020d8db7d81d6cff39d41556b129c44b1feb76279c71ff6",
        ),
    ] {
        let bytes = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("schemas")
                .join(filename),
        )
        .expect("schema is readable");

        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected_hash);
    }
}

#[test]
fn validates_program_workouts_against_the_external_template_schema() {
    let program = json!({
        "name": "Program",
        "weeks": [{
            "workouts": [{
                "name": "Day 1",
                "exercises": [{
                    "exercise": { "name": "Squat" },
                    "sets": [{ "targetWeight": 100 }]
                }]
            }]
        }]
    });

    let result = validate(DocumentKind::Program, &program);

    assert!(!result.valid);
    assert!(!result.errors.is_empty());
}

#[test]
fn validates_openweight_format_and_numeric_constraints() {
    let invalid_date = json!({
        "date": "not-a-date",
        "exercises": [{ "exercise": { "name": "Squat" }, "sets": [{ "reps": 5 }] }]
    });
    let fractional_reps = json!({
        "date": "2024-01-15T09:00:00Z",
        "exercises": [{ "exercise": { "name": "Squat" }, "sets": [{ "reps": 5.5 }] }]
    });
    let decimal_rpe = json!({
        "date": "2024-01-15T09:00:00Z",
        "exercises": [{ "exercise": { "name": "Squat" }, "sets": [{ "reps": 5, "rpe": 7.5 }] }]
    });
    let space_separated_date_time = json!({
        "date": "2024-01-15 09:00:00Z",
        "exercises": [{ "exercise": { "name": "Squat" }, "sets": [{ "reps": 5 }] }]
    });

    assert!(!validate(DocumentKind::WorkoutLog, &invalid_date).valid);
    assert!(!validate(DocumentKind::WorkoutLog, &fractional_reps).valid);
    assert!(validate(DocumentKind::WorkoutLog, &decimal_rpe).valid);
    assert!(!validate(DocumentKind::WorkoutLog, &space_separated_date_time).valid);
}

#[test]
fn accepts_extensions_at_nested_object_levels() {
    let workout = json!({
        "date": "2024-01-15T09:00:00Z",
        "app:workout": true,
        "exercises": [{
            "app:exercise-log": true,
            "exercise": { "name": "Squat", "app:exercise": true },
            "sets": [{ "reps": 5, "app:set": true }]
        }]
    });

    let result = validate(DocumentKind::WorkoutLog, &workout);

    assert!(result.valid, "extensions failed: {:?}", result.errors);
    assert!(result.errors.is_empty());
}

fn fixtures_in(directory: &str) -> Vec<(std::path::PathBuf, Value)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(directory);
    let mut fixtures = fs::read_dir(&path)
        .expect("fixture directory exists")
        .map(|entry| entry.expect("fixture directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    fixtures.sort();

    fixtures
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("fixture is readable");
            let fixture = serde_json::from_str(&source).unwrap_or_else(|error| {
                panic!("fixture {} is valid JSON: {error}", path.display())
            });

            (path, fixture)
        })
        .collect()
}

fn fixture_paths() -> BTreeSet<String> {
    [
        "invalid",
        "lifter-profiles",
        "programs",
        "workout-logs",
        "workout-templates",
    ]
    .into_iter()
    .flat_map(|directory| {
        fs::read_dir(
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
        .map(move |path| {
            format!(
                "{directory}/{}",
                path.file_name().expect("filename").to_string_lossy()
            )
        })
    })
    .collect()
}

fn expected_fixture_paths() -> BTreeSet<String> {
    [
        "invalid/distance-without-unit.json",
        "invalid/empty-exercises.json",
        "invalid/empty-sets.json",
        "invalid/invalid-unit.json",
        "invalid/missing-date.json",
        "invalid/missing-exercise-name.json",
        "invalid/missing-exercises.json",
        "invalid/profile-invalid-formula.json",
        "invalid/profile-invalid-height-unit.json",
        "invalid/profile-invalid-rep-max.json",
        "invalid/profile-invalid-sex.json",
        "invalid/profile-missing-exported-at.json",
        "invalid/program-empty-weeks.json",
        "invalid/program-missing-weeks.json",
        "invalid/target-weight-without-unit.json",
        "invalid/template-empty-exercises.json",
        "invalid/template-missing-name.json",
        "invalid/template-weight-without-unit.json",
        "invalid/weight-without-unit.json",
        "lifter-profiles/full-featured.json",
        "lifter-profiles/imperial-units.json",
        "lifter-profiles/minimal.json",
        "programs/531-bbb.json",
        "programs/minimal.json",
        "workout-logs/bodyweight-workout.json",
        "workout-logs/carries-and-distance.json",
        "workout-logs/full-featured.json",
        "workout-logs/hypertrophy-session.json",
        "workout-logs/minimal.json",
        "workout-logs/programmed-workout.json",
        "workout-logs/simple-strength.json",
        "workout-logs/superset-workout.json",
        "workout-logs/tempo-training.json",
        "workout-logs/timed-exercises.json",
        "workout-templates/full-featured.json",
        "workout-templates/minimal.json",
        "workout-templates/percentage-based.json",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn kind_for_invalid_fixture(path: &Path) -> DocumentKind {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("fixture filename is UTF-8");

    if name.starts_with("template-") {
        DocumentKind::WorkoutTemplate
    } else if name.starts_with("program-") {
        DocumentKind::Program
    } else if name.starts_with("profile-") {
        DocumentKind::LifterProfile
    } else {
        DocumentKind::WorkoutLog
    }
}
