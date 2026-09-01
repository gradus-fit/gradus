# Vault Model

Each user vault has the following logical file structure. All files are encrypted before storage or upload.

```text
<user-vault>/
├── training-programs/
│   └── <program-resource-id>.ics
├── planned-workouts/
│   └── <workout-resource-id>.fit
├── recorded-activities/
│   └── <activity-resource-id>.fit
└── recorded-routes/
    └── <route-resource-id>.gpx
```

| Path | File | Stores |
|---|---|---|
| `training-programs/<program-resource-id>.ics` | iCalendar | One static training-program calendar. |
| `planned-workouts/<workout-resource-id>.fit` | FIT workout | One planned structured workout. |
| `recorded-activities/<activity-resource-id>.fit` | FIT activity | One completed activity recording. |
| `recorded-routes/<route-resource-id>.gpx` | GPX route | One optional route recorded during a completed activity. |

Every resource ID is a client-generated UUIDv7. Training-program calendars reference planned workouts, recorded activities, and optional recorded routes by resource ID, not by file path. See `docs/training_program.md` for the calendar format.
