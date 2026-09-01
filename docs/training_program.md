# Training Program Format

## Status

Draft — this is the canonical plaintext representation for a static Gradus training program. It defines data that Clients create and parse locally before encryption; the Server stores every resource as opaque encrypted bytes.

## Purpose and scope

A training program is a finite calendar of planned sessions. Each session references one required FIT **structured-workout** resource. After completion, it may reference the FIT activity and GPX route recorded during that session.

```text
training-program ICS resource
  └── VEVENT (one planned session or a finite recurrence)
        ├── ATTACH URI → planned-workout FIT resource
        ├── ATTACH URI → optional recorded-activity FIT resource
        └── ATTACH URI → optional recorded-route GPX resource

planned-workout FIT resource → sent to a device for execution
recorded-activity FIT resource → attached to the completed event as its result
recorded-route GPX resource → attached to the completed event as its route result
```

The calendar is the schedule and the event-to-result association. The planned-workout FIT file is the execution definition. A recorded-activity FIT file is the result. The planned FIT file and the recorded FIT file are separate immutable resources; a client must never append recorded data to, or overwrite, a planned-workout FIT file.

This format deliberately does not contain adaptive rules or completion-gated progression. A recorded-activity attachment records a completed result but never changes the static schedule. When the schedule needs to change, Gradus recalculates and saves a new static calendar revision.

## Resources and identifiers

Every vault file is a resource with a client-generated [UUIDv7](https://www.rfc-editor.org/rfc/rfc9562) resource ID. IDs use their canonical lowercase, hyphenated string form. A resource ID identifies the resource independently of where or how it is stored; paths are never identifiers or links.

This format uses these logical resource types:

| Logical type | Plaintext content | Purpose |
|---|---|---|
| `training-program` | UTF-8 iCalendar (`.ics`) | The static program schedule. |
| `planned-workout` | FIT (`.fit`) whose FIT file type is `workout` | One device-executable session. |
| `recorded-activity` | FIT (`.fit`) whose FIT file type is `activity` | A completed, recorded session. |
| `recorded-route` | GPX (`.gpx`) | One optional route recorded during a completed session. |

The encryption layer binds each resource to its opaque stable resource-type ID, resource ID, and revision; its registry of binary type IDs is outside this document. The logical names above are for Clients and documentation.

### Resource attachment URI

A planned event references a vault resource with a standard iCalendar `ATTACH` property whose URI is exactly:

```text
gradus://resource/<uuidv7-resource-id>
```

For example:

```ics
ATTACH;FMTTYPE=application/octet-stream:gradus://resource/0195d145-24c5-7b93-924e-9b74ca8f7f1a
```

`gradus` is an application URI scheme, not a network protocol. Clients resolve it only by looking up the referenced resource ID in the currently unlocked vault. They must not make a network request, dereference another URI scheme, or treat it as a filesystem path.

FIT has no suitable registered media type for this purpose, so both FIT attachments use `FMTTYPE=application/octet-stream`. The required planned-workout attachment must reference a `planned-workout` resource that parses as a FIT file whose file type is `workout`. An optional recorded-activity attachment must reference a `recorded-activity` resource that parses as a FIT file whose file type is `activity`.

An optional recorded-route attachment uses `FMTTYPE=application/gpx+xml`. It must reference a `recorded-route` resource and parse as GPX. It is created only after the activity is completed; it is not a route sent to the device.

## iCalendar document

The plaintext program file conforms to [RFC 5545 iCalendar](https://www.rfc-editor.org/rfc/rfc5545). It is UTF-8 and uses RFC 5545 content-line folding and CRLF line endings.

The root component is exactly one `VCALENDAR` with:

```ics
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Gradus//Training Program 1.0//EN
CALSCALE:GREGORIAN
```

`METHOD` is omitted: a training program is stored calendar data, not an iTIP scheduling message. Clients may include standard display properties such as `X-WR-CALNAME`, but parsing must not depend on them.

### Planned-session events

Every planned session is represented by a `VEVENT` and has all of the following:

- `UID` — globally unique event ID. Gradus emits `urn:uuid:<uuidv7>`.
- `DTSTART;VALUE=DATE` — the local calendar day on which the user should train. A program intentionally has no time-zone or time-of-day requirement.
- `SUMMARY` — a human-readable session title.
- exactly one `ATTACH;FMTTYPE=application/octet-stream` with a `gradus://resource/<uuidv7>` URI for its planned workout; and
- zero or one further `ATTACH;FMTTYPE=application/octet-stream` with a `gradus://resource/<uuidv7>` URI for its recorded activity; and
- zero or one `ATTACH;FMTTYPE=application/gpx+xml` with a `gradus://resource/<uuidv7>` URI for its recorded route.

A one-day event omits `DTEND`; it is an all-day session on its `DTSTART` date. A multi-day event is not a supported training session.

`DESCRIPTION` is optional but strongly recommended. It contains a human-readable summary of the workout so an exported program remains useful in ordinary calendar software that cannot open FIT attachments. Values must use normal RFC 5545 text escaping.

The calendar must not contain a planned-session `VEVENT` without its FIT workout attachment, multiple planned-workout attachments, multiple recorded-activity attachments, multiple GPX route attachments, a recorded-route attachment without a recorded-activity attachment, or an attachment to a resource type other than `planned-workout`, `recorded-activity`, or `recorded-route`.

### Recurrence and static plans

A repeating session may use RFC 5545 `RRULE`. It is still static only when the rule is finite: `RRULE` must include either `COUNT` or `UNTIL`. Every generated occurrence of that `VEVENT` uses the same planned-workout FIT resource.

Use separate one-off events for progressive sessions with different FIT definitions. A changed occurrence of a recurrence uses the normal RFC 5545 `RECURRENCE-ID` override and may reference a different planned-workout resource. A recorded activity and recorded route for a recurring occurrence are attached only to that occurrence's `RECURRENCE-ID` override, never to the recurring master event. `RDATE`, `EXDATE`, and `RECURRENCE-ID` remain RFC 5545 calendar semantics; they do not encode adaptive behavior.

A Client that recalculates a program creates a new encrypted revision of its `training-program` resource. For the same planned session, it retains the event `UID` and increments `SEQUENCE`; a newly added session receives a new `UID`. Recording a completed activity or route also creates a calendar revision and increments the completed event's `SEQUENCE`, but does not change its schedule, planned workout, or historical result resources.

## Example

This program schedules an early run/walk workout. The attached resource is a `planned-workout` FIT file containing the warm-up, run/rest interval block, and cool-down steps.

```ics
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Gradus//Training Program 1.0//EN
CALSCALE:GREGORIAN
BEGIN:VEVENT
UID:urn:uuid:0195d143-3588-7c73-9f27-83cf4a194b32
DTSTART;VALUE=DATE:20260406
SUMMARY:Week 1 · Run/walk intervals
DESCRIPTION:Warm up; 10 × (1 min run, 1 min recovery); cool down.
ATTACH;FMTTYPE=application/octet-stream:gradus://resource/0195d145-24c5-7b93-924e-9b74ca8f7f1a
END:VEVENT
END:VCALENDAR
```

The resource ID in `ATTACH` is not a filename. On an unlocked device it resolves to the independently encrypted `planned-workout` resource with that ID.

## Client parsing and execution

When opening a training program, a Client must:

1. decrypt the `training-program` resource using its resource binding;
2. validate and parse its iCalendar bytes as RFC 5545;
3. expand finite recurrence rules and apply their RFC 5545 overrides;
4. validate each planned event's required properties and resolve its `ATTACH` URIs only within the same unlocked vault;
5. decrypt the referenced `planned-workout` resource using that resource's own binding and parse it as a FIT workout;
6. when a recorded activity is attached, decrypt the referenced `recorded-activity` resource using its own binding and parse it as a FIT activity; and
7. when a recorded route is attached, decrypt the referenced `recorded-route` resource using its own binding and parse it as GPX.

A malformed calendar, invalid/duplicate event UID, unbounded recurrence, missing or ambiguous attachment, invalid resource URI/UUIDv7, missing resource, wrong resource type, failed decryption, invalid FIT file type, or invalid GPX file makes the affected program/session unavailable. Clients must surface a non-sensitive error and must not guess a path, fetch an attachment, or substitute a workout.

To execute a session, the Client sends the referenced planned-workout FIT bytes to a compatible device. After the session, it imports or creates a **new** `recorded-activity` FIT resource and, when available, a new `recorded-route` GPX resource. It adds their resource URIs as result attachments to the completed event and saves a new calendar revision. This does not alter the planned FIT file or replace its attachment.

## Encryption and server storage

The `.ics` calendar, every `.fit` file, and every `.gpx` file are encrypted independently before upload. The Server receives resource identity/revision metadata and ciphertext only; it cannot read calendar titles, dates, attachment references, FIT steps, activity data, or route data. In particular, resource IDs appearing inside `ATTACH` are encrypted as part of the calendar plaintext.

The Server preserves encrypted bytes exactly. It must not parse, expand recurrence, resolve attachments, inspect FIT or GPX data, or create a relationship from a program resource to a workout, activity, or route resource.

## Non-goals

This format does not define:

- an adaptive or conditional training language;
- completion policy, skipped-session, perceived-effort, injury, or progression state;
- device transfer protocols or Garmin-specific import behavior;
- route planning, navigation, or activity analysis; or
- a server-visible index of program dates, workouts, routes, or activities.

## References

- RFC 5545: [Internet Calendaring and Scheduling Core Object Specification](https://www.rfc-editor.org/rfc/rfc5545)
- RFC 9562: [Universally Unique IDentifiers (UUIDs)](https://www.rfc-editor.org/rfc/rfc9562)
- Garmin: [Flexible and Interoperable Data Transfer (FIT) SDK](https://developer.garmin.com/fit/overview/)
- Topografix: [GPX 1.1 schema](https://www.topografix.com/GPX/1/1/)
- `docs/vault_model.md` — logical vault layout for these resources
