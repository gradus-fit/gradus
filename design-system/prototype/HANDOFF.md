# Activity UI handoff to Flutter

These four HTML pages are visual studies, not production UI or a source of Flutter
widgets: [activity archive](activity-archive.html), [run detail](activity-detail.html),
[strength detail](gym-activity-detail.html), and [workout logging](gym-workout.html).
Use [the design manifesto](../MANIFESTO.md) for the product voice: calm, precise,
private, and focused on comparison with one's past self. The values and routes in
the studies are illustrative. Flutter will render mobile **and** web; do not embed
these pages or translate their HTML/CSS line for line.

## One owner for each shared pattern

A repeated visual pattern must have **one Flutter implementation**. Screens supply
content and state; they must not copy the pattern's padding, typography, border,
layout, or interaction logic. This matters even where the studies differ today:
treat differences in duplicated CSS as prototype drift unless this document names
a variant. Build the shared pieces inside the owning Client feature module first;
move a stable, app-wide primitive to a separate Client UI module only when another
feature needs it. `client/app` remains the composition root for routes and
construction, not the owner of feature widgets.

| Shared piece | Contract for Flutter | Used by |
| --- | --- | --- |
| `GradusHeader` | **One header widget**: centered Gradus brand, one height, gutters, bottom rule and striped accent. Inputs are a leading back action or context label and a trailing action or status. Back is a real navigation action, never a decorative arrow. Reserve symmetric space so the brand stays centered even when the sides have different widths. Use the same focus, tap area and text-scaling rules everywhere. | All four pages. Archive: Activity / 2026; details: Archive / share affordance; logging: Training plan / elapsed time. Share is only actionable when implemented. |
| `ActivityHero` + `MetricStrip` | Kicker, title, supporting date/location or start time, optional decorative artwork, and a row of labeled metrics. Artwork selection, content and metrics are inputs; metric typography, dividers, responsive wrapping and spacing have one owner. Keep art out of the metric row. Hero height can vary with content, not with a different copy of the widget. | All four: archive totals, completed run/strength totals, workout plan and progress. Only logging adds an exercise/set progress indicator. |
| `SectionHeading` | One label, title and optional trailing context layout. Consistent type hierarchy and gutters; context wraps or moves below when space is tight. | Archive list and detail/logging sections. |
| `SurfaceCard` | Shared paper/white surface, border, offset shadow and inner spacing; use deliberate variants such as emphasized/current, not separate CSS clones. | Archive rows, coach briefs, chart panels, exercise cards, route map. |
| `CoachInsightCard` | Title, explanatory sentence, two labeled comparison values, methodology line. A shared **completed-activity** card with run/strength data as inputs. | Both completed detail pages. |
| `ComparisonChart` | One panel for metric choices, value + caption, legend, chart, axis labels, takeaway and status. Switching metrics changes the plotted data **and** the spoken/visible description as one state update. | Run detail (pace/heart rate/elevation); strength detail (volume/reps/RPE). |
| `ActivityDetails` + `PrivacyNotice` | Consistent expandable label/value details and consistent privacy presentation. Text depends on state: a completed saved activity differs from an unsaved workout prototype. | Both detail pages; privacy notice also in logging. |

The workout's expandable **coach tips** and editable **exercise/set log** are
separate feature widgets: they have different information and behaviour from
`CoachInsightCard` or an archive activity row. Reuse `SurfaceCard`, headings and
field styling, not a universal card with dozens of flags. Likewise, run splits
and strength sets can share table spacing and typography without sharing a
fixed column schema.

### Route map: reusable view, not a hard-coded picture

The run detail's illustrated map is currently CSS scenery plus one fixed SVG
path (`activity-detail.html`, “The course”). In Flutter, make a `RouteMapView`
whose input is a route (ordered coordinates, bounds or camera framing, start and
finish), optional place labels, optional overlays such as ascent, and an accessible
route description. The widget owns route colour, line weight, markers, legend,
clipping, framing and label placement. A screen must not bake in this particular
Riverside loop or create a second map implementation for a new activity.

The HTML's roads, water and route are **illustrative**, not geographic data or a
chosen map provider. Decide the production basemap/renderer and what to show
when route data is unavailable before implementing the Flutter map. Preserve
route contrast and start/finish distinction on mobile and web; provide a text
summary so the route is understandable without seeing the image. Keep mapping
separate from `ComparisonChart`: they share a card surface, not a data model.

## Page compositions

- **Archive:** `GradusHeader` → `ActivityHero`/`MetricStrip` → month
  `SectionHeading` → activity list rows. Each row owns its date, type, title,
  relevant summary and destination; do not assume every activity has a distance.
- **Run detail:** shared header/hero → `CoachInsightCard` → course heading and
  `RouteMapView` → `ComparisonChart` → splits → `ActivityDetails` → privacy.
- **Strength detail:** shared header/hero → `CoachInsightCard` → session/exercise
  list → `ComparisonChart` → set summary → `ActivityDetails` → privacy.
- **Workout logging:** shared header/hero with progress → expandable coach tips
  → editable exercise/set cards → optional session note → privacy → persistent
  finish action. State belongs to the logging feature: mark sets done only with
  valid actual results, update progress, surface an incomplete set when finishing,
  and distinguish a finished session from a persisted one.

## Visual and interaction rules to carry forward

- Use the canonical [design tokens](TOKENS.md) for palette, typography,
  spacing, surfaces, component variants and responsive starting points. Screens
  must not copy CSS values into their own Flutter styles. Respect safe areas,
  text scaling, keyboard insets and usable web widths. The archive artwork must
  fit above the metric strip, including its feet.
- Give interactive rows, tabs, checkboxes, back actions and finish controls clear
  focus, pressed, selected and disabled states and adequate touch targets. Do not
  convey completion or comparison solely by colour. Keep loading/empty/error
  presentations as explicit Flutter design decisions; these HTML samples show
  populated states only.
- Preserve the distinction between **target** and **actual** results, and label
  units explicitly (kg, reps, seconds, pace). A changing chart needs an accessible
  summary; a route needs a text alternative. Avoid treating static prototype
  copy such as the logging rest-time hint as a live timer.
- Use objective copy. Do not claim a workout is saved or encrypted until the
  production flow actually persists it; the prototypes do not save data.

## Source of truth during implementation

[TOKENS.md](TOKENS.md) is the approved design decision record while these pages
are experiments. Once Flutter implements the theme and shared widgets, their
typed Dart values become the production source of truth; update the document to
reflect decisions, but do not maintain a second independent set of production
values in Markdown. Because Flutter serves mobile and web, there is no need for
a JSON token pipeline just to support both targets. If a design tool or another
UI platform later needs the same tokens, consider DTCG-format JSON as the single
machine-readable source and generate the Dart values from it. Do not maintain
JSON and Dart tokens by hand in parallel.

## Implementation and consistency checks

Use a small shared-widget gallery or tests with the **same header** rendered in
archive, run detail, strength detail and logging configurations. Check brand
centering and responsive layout at narrow phone width, larger phone width,
text scaling and desktop web width. Do the same for metric strips with short and
long labels, chart choices with different data, and `RouteMapView` with two
different routes; one fixture must not define the component. Include widget
interaction/semantics tests for back navigation, tabs, coach disclosure,
completion states and the map's accessible summary. Feature screens should test
their composition, not duplicate the internals of shared widgets.

The artwork is prototype-only until provenance and redistribution terms are
confirmed; see [asset notes](../README.md#image-attribution). In particular,
`assets/disk.png` has no documented provenance in that list. The HTML remains a
visual reference; the Flutter widgets and theme become the production source of
truth once implemented.
