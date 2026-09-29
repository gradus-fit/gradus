# Modern Palaestra design tokens

Canonical visual decisions for the four [activity UI studies](HANDOFF.md). These
values are taken from `activity-archive.html`, `activity-detail.html`,
`gym-activity-detail.html` and `gym-workout.html`. The studies are approved visual
references and keep their own CSS; this document resolves their small differences
**for a single Flutter theme and shared widgets**. A screen supplies content and
state, not its own version of a shared token. Sizes below are logical pixels for
Flutter (the source CSS uses px and rem at the browser's default 16 px base).
Use scalable text rather than forcing fixed heights when text grows.

## Colour

| Foundation | Value | Semantic use |
| --- | --- | --- |
| `canvas` | `#D7CDBD` | Surrounding desktop/web canvas. |
| `surface` | `#F4EFE5` | Screen background and hero base. |
| `surfaceDeep` | `#EBE2D3` | Hero gradient end and quiet exercise headers. |
| `surfaceRaised` | `#FFFDF8` | Cards and form fields. |
| `textPrimary` | `#29251F` | Primary text and data. |
| `textSecondary` | `#6D655A` | Supporting text, labels and metadata. |
| `border` | `#C9BCAA` | Dividers and input borders. |
| `borderStrong` | `#8A7C68` | Masthead and metric-strip rules. |
| `accent` | `#A94D35` | Terracotta: emphasis, routes and primary actions. |
| `accentText` | `#763523` | Terracotta text on pale surfaces. |
| `accentSoft` | `#EAD4C8` | Terracotta-tinted interactive surfaces. |
| `comparison` | `#2E6871` | Blue: chart/range accent and visible focus. |
| `comparisonSoft` | `#C9DCDA` | Chart range fill. |
| `complete` | `#6F693B` | Olive: completed/logged status. |
| `completeSoft` | `#DCD8BD` | Olive-tinted comparison range. |

Treat the colour names as **roles**, not activity types: run and strength share
terracotta for emphasis, while chart colours identify the selected metric. The
blue chart is not a universal “running” colour. Current/active uses terracotta;
completed uses olive; pending uses secondary text. Invalid fields use terracotta
**plus an explanatory message**. Focus uses blue, a 3 px outline and visible
spacing. Check contrast on actual rendered text sizes and states before shipping;
colour alone must not carry meaning.

The illustrated map has its own `mapSurface` (`#D7D2C4`), terracotta route,
white/terracotta start-and-finish markers, and a pale label surface. Its painted
water, roads and place labels are **map treatment**, not global colour tokens.
Do not copy literal basemap colours into other screens. Chart series tints are
metric data/styles; the shared chart widget owns their selection and legend.

## Typography

| Role | Canonical starting style | Where |
| --- | --- | --- |
| `display` | Georgia/serif, weight 500, responsive ~50–78 px, tight tracking (~−.075 em), line height ~.84–.88 | Hero title. |
| `sectionTitle` | Georgia/serif, weight ~550, 28 px, tracking −.045 em, line height 1 | Archive/detail/logging section headings. |
| `brand` | Georgia/serif, weight 700, 20 px, tracking −.04 em | Every masthead; centered. |
| `cardTitle` | Georgia/serif, weight 600, ~19–20 px | Activity/exercise titles. |
| `metricValue` | Georgia/serif, weight 600; detail 20–27 px, compact/archive 19–24 px, dense logging ~18 px | Metric strip; compact and dense are explicit variants of one component. |
| `body` | System sans, ~14 px with comfortable line height | Explanatory text. |
| `meta` | System sans, ~13–14 px | Dates, locations and support text. |
| `eyebrow` | System sans, weight 800, ~10–11 px, uppercase, tracking .08–.16 em | Kicker, section label and metric label. |

Use text **roles**, not separate archive/run/gym heading styles: the studies'
1.78/1.8 rem section titles and .65/.67/.68 rem labels become one style each.
Density variants belong only where content warrants them (metric strip, set
log). Scale text with user settings; allow labels and values to wrap rather than
clip. Georgia is used by the HTML studies, but its availability varies by
platform: the Flutter implementation must use a consistently available or
bundled/licensed serif, not rely on a platform-specific fallback.

## Layout, surfaces and controls

| Token / shared rule | Canonical decision | Source of variation |
| --- | --- | --- |
| `contentMaxWidth` | 560 px centered on wide displays | All four screens. This is a content constraint, not a fixed phone width. |
| `pageGutter` | 24 px; 18 px on narrow phones (at or below ~390 px in the studies) | Section and hero content. Respect safe areas. |
| `headerGutter` | 20 px; 16 px on narrow phones | Same masthead in all four screens. |
| `headerMinHeight` | 66 px plus any height needed for scaled text/safe area | All mastheads. |
| `headerStripe` | 3 px high, 4 px above the masthead bottom, matching horizontal insets | All mastheads; terracotta/ink repeating motif. |
| `sectionTopGap` | 42 px standard; 32 px for the archive's denser list transition | Detail CSS uses 42, logging 40, archive 32. Flutter uses standard 42 rather than a logging-only 40. |
| `cardGap` | 16 px default | Archive uses 14, logging 18: one rhythm for general cards; denser grids may have their own layout. |
| `hairline` | 1 px | Rules, cards and fields. |
| `cardBorder` | `#B7AA97`, 1 px | Archive cards, route map, exercise cards. Coach brief uses a separate warm border and 5 px terracotta left accent. |
| `cardShadow` | Offset 4 px right / 5 px down, no blur, `rgb(103 77 46 / 10%)` | Repeated cards, coach and map. |
| `screenShadow` | `0 22px 55px rgb(67 53 34 / 18%)` | Desktop prototype frame only; not a mobile elevation. |
| `chartPlotHeight` | 174 px | Both detail charts. |
| `routeMapHeight` | 286 px starting point | Run detail map; allow responsive resizing without cropping route markers. |
| `minTouchTarget` | At least 44 × 44 px (tabs currently 46 px tall; finish action 48 px) | Increase the effective tap area of the prototype's small set checkboxes in Flutter. |

Spacing primitives: **4, 8, 12, 16, 20, 24, 32, 42 px**. Use them for new
shared layouts; preserve deliberate content-specific card padding instead of
trying to force every surface onto the same interior grid. The archive's 82 px
hero-meta gap is an **artwork clearance**, not a reusable spacing token: the
Flutter hero must lay out its artwork above the metric strip based on available
space, not copy that fixed gap. Likewise, archive/detail/logging heroes may have
different total heights while sharing title, metric and artwork rules.

## Variants and ownership

- `GradusHeader` owns brand position, gutters, striped rule and type **once**.
  Archive uses context/year, completed details use back/action, logging uses
  back/timer. Those are data/slot variants, not separate header styles. Prefer
  the archive's shrink-safe symmetric side tracks so long labels do not push the
  brand off center; provide accessible back/action targets.
- `MetricStrip` owns divider, value and label styles. Use `detail`, `compact`
  (archive), or `dense` (logging) presentation based on content density; do not
  select sizing by activity type. Run and strength detail use the **same**
  `detail` variant. Values must wrap safely under text scaling.
- `SectionHeading` uses one type and gutter system. Its optional right context
  wraps/moves below rather than colliding with the title.
- `SurfaceCard` owns the border and shadow; emphasized/complete are explicit
  states. The route map and coach brief keep their distinct interior treatments.
- `RouteMapView` owns route line, markers, legend, labels and framing. Map data
  and accessible summary are inputs; changing the activity must not introduce
  new screen-local route colours or a second map component.
- `ComparisonChart` owns the panel, metric tabs, selected-state visuals,
  legend, line/range/marker and description updates. The two detail studies'
  chart styling is already effectively identical.

The studies also differ in the prototype desktop frame (archive has padding on
all sides, details/logging only at the top), hero title sizing, and focus offset
(2 vs 3 px). These are **not** new page tokens: use one frame policy for Flutter
web, one title role, and one accessible focus rule. When an approved visual
needs an exception, name and document a component variant rather than copy a
magic number into a screen.

Implement colours/text in the Flutter theme and any extra component tokens in a
small theme extension or equivalent within the owning Client module. Keep
component-specific dimensions with the shared widget, not in a screen. The
[handoff](HANDOFF.md) defines widget contracts and screen composition; the
studies remain visual references. No production Flutter code or shared CSS is
created by this token decision document.
