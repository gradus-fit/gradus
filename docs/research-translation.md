# Translation Research

## Status

Research and proposed direction. No localization implementation exists today: `client/app` has no ARB catalog, `l10n.yaml`, generated localization API, or Flutter localization dependencies. The current Server is FastAPI only; it has no HTML-template or translation subsystem.

## Decision

Use **Application Resource Bundle (ARB)** files as the one canonical, version-controlled catalog format for Gradus UI translations.

ARB is Flutter's supported input to `flutter gen-l10n`, is JSON-based, uses ICU MessageFormat for variables/plurals/selects, and is directly supported by Weblate. A future server-rendered HTML application can read the exact same files with an ARB loader and ICU MessageFormat-compatible formatter.

Do not make GNU gettext `.po`/`.pot` files canonical. They are open, well-supported Weblate formats, but Flutter's official generator does not consume them. Making PO canonical would require a third-party Flutter runtime or a PO-to-ARB conversion pipeline. That creates an avoidable compatibility boundary for plural rules, `select` expressions, placeholders, and translator metadata.

## Shared catalog layout

When implementation begins, keep source ARB files in one repository location that both consumers can read, rather than duplicating them per application. For example:

```text
translations/
  app_en.arb          # required source/fallback catalog
  app_pt_BR.arb
  app_es.arb
client/app/
  l10n.yaml           # points Flutter gen-l10n at ../../translations
server/                # future HTML application reads ../../translations
```

`translations/` is source content, not generated output and not a Client or Server module. Generated Flutter localization Dart files remain generated application implementation details.

The exact language set is a product decision. Use BCP 47 language tags at the product boundary and in web URLs, such as `en` and `pt-BR`. Flutter ARB filenames conventionally represent a region with an underscore (`app_pt_BR.arb`); the web locale resolver maps `pt-BR` to that catalog name.

### Catalog rules

- `app_en.arb` is the complete source and fallback catalog.
- Every translation uses the same stable, semantic camelCase key as English. Keys must be valid Dart identifiers because Flutter generates a Dart getter/method for each key. For example, use `startWorkout`, not the English phrase or `workout.start`.
- Every user-visible UI string is a catalog entry; do not concatenate translated fragments in code.
- Keep values as plain text. HTML templates own HTML structure and escape values normally; translated strings must never be injected as untrusted raw HTML.
- Define a clear description and every placeholder in the `@<key>` metadata entry. This gives translators context and lets Flutter generate typed APIs.
- Use ICU MessageFormat for variable, plural, and select messages. The runtime must format it using the current locale's CLDR rules.
- Locale-independent identifiers, API fields, encrypted vault content, and user-entered data are not UI translations.

Example `app_en.arb`:

```json
{
  "@@locale": "en",
  "startWorkout": "Start workout",
  "@startWorkout": {
    "description": "Button that starts a workout session"
  },
  "exerciseCount": "{count, plural, =0{No exercises} one{1 exercise} other{{count} exercises}}",
  "@exerciseCount": {
    "description": "Number of exercises in a workout",
    "placeholders": {
      "count": { "type": "num" }
    }
  }
}
```

A Portuguese catalog contains the same message keys and matching placeholders, with Portuguese values.

## Flutter application

Flutter's official localization workflow is `flutter_localizations` + `intl` + `flutter gen-l10n`.

1. Add `flutter_localizations` (from the Flutter SDK) and `intl` to `client/app` dependencies, and enable `flutter: generate: true`.
2. Add `client/app/l10n.yaml`, setting `arb-dir` to the shared catalog directory, `template-arb-file` to `app_en.arb`, and an output localization file.
3. Run `flutter gen-l10n` as part of the normal build/check workflow. It validates the ARB catalogs and generates the typed `AppLocalizations` API.
4. The Flutter composition root registers `AppLocalizations.delegate`, `AppLocalizations.localizationsDelegates`, and `AppLocalizations.supportedLocales` on `MaterialApp`.
5. Widgets obtain localized text from `AppLocalizations.of(context)`; they do not embed English UI text.

Conceptually, a widget calls a generated method:

```dart
Text(AppLocalizations.of(context)!.exerciseCount(exerciseCount))
```

Flutter selects the device/application locale and applies its normal locale fallback. The app should offer an explicit user language setting in addition to the device default when product requirements call for it.

## Future server-rendered HTML application

This design also works with plain HTML and htmx. There is no browser-side ARB runtime required for server-rendered text.

```text
GET /pt-BR/workouts
  -> resolve the requested locale
  -> select and load the matching ARB catalog
  -> look up and ICU-format each message in the server template
  -> return Portuguese HTML
```

The future FastAPI HTML layer would provide a request-scoped translator to its template engine:

```html
<button>{{ t("startWorkout") }}</button>
<p>{{ t("exerciseCount", count=exercise_count) }}</p>
```

For `exercise_count = 3`, the translated response contains ordinary final HTML, for example:

```html
<button>Iniciar treino</button>
<p>3 exercícios</p>
```

### Required server translation service

The future Server implementation needs one small translation service that:

1. loads and parses ARB JSON at startup or from a safe cache;
2. removes metadata entries whose keys begin with `@`;
3. resolves the selected catalog and falls back first to `app_en.arb`, then to the key only as a development-safe last resort;
4. formats message text with an **ICU MessageFormat-compatible** Python formatter using the selected locale; and
5. exposes a request-scoped `t(key, **values)` function to templates and HTML-producing application code.

Python `str.format()` is not sufficient: it cannot evaluate the ICU plural/select expression used in ARB. Select and validate a maintained ICU MessageFormat-compatible Python dependency during implementation, including compatibility with the Server's supported Python version. Cache parsed/compiled catalog messages, but never cache a rendered response without accounting for its locale.

### Locale resolution and htmx

Prefer locale-prefixed URLs as the authoritative locale:

```text
/en/workouts
/pt-BR/workouts
/es/workouts
```

They are bookmarkable, shareable, and suitable for indexing and cache keys. A language selector redirects to the equivalent URL in the chosen locale. A saved account preference or cookie may choose the initial redirect; `Accept-Language` is only a fallback for first-time visitors.

HTMX makes regular HTTP requests. A fragment request therefore follows the same locale resolver and returns a fragment in the same language:

```html
<button hx-get="/pt-BR/workouts/42/exercises">
  Ver exercícios
</button>
```

No client-side `t()` function or catalog download is necessary for server-rendered content. If future client-side JavaScript needs visible messages (for example, a validation error), the server must either render the text into the page/fragment or deliberately add a JavaScript ARB + ICU formatter for that isolated use case.

Responses cached by a CDN or reverse proxy must be partitioned by locale. If locale comes from a cookie or `Accept-Language`, configure the appropriate `Vary` behavior; locale-prefixed paths make this substantially simpler. HTML fragments must also vary by whether the response is a full page or an htmx fragment.

## Weblate workflow

Weblate supports ARB directly, including ICU MessageFormat checks. Configure one Git-backed Weblate component with:

- `app_en.arb` as the monolingual base/source catalog; and
- `app_<language>.arb` as its translated catalogs.

Weblate should contribute changes through the repository's normal branch/PR workflow. Engineers add or alter English source entries; Weblate presents the resulting work to translators and commits translated ARB updates. Both Flutter and the future HTML server use those updates directly.

Translator descriptions in ARB metadata are required operational context. Screenshots/context in Weblate should be added for ambiguous labels such as `Save`, `Set`, or short button text.

## Validation requirements

Before accepting a catalog change:

- parse every ARB file as JSON;
- run `flutter gen-l10n` and the Flutter localization tests;
- verify each non-English catalog has the source keys and matching placeholder names/types;
- parse/format representative ICU messages for every supported locale in the server translation tests;
- test plural cases (`0`, `1`, and another count), locale fallback, and unsupported-locale handling;
- test a normal HTML response and an htmx fragment in at least two locales; and
- ensure template rendering escapes interpolated user data and never treats a translated string as raw HTML.

## Implementation sequence

This document is research, not an implementation plan that authorizes cross-module changes. When implementation is requested, scope the work according to the repository's module rules.

1. Establish the shared ARB catalogs and Weblate component.
2. Add Flutter `gen-l10n` support to the Client composition root and migrate its visible text.
3. When the HTML application exists, implement and test the Server ARB/ICU translation service and template integration.

The Flutter and Server consumers should be independently deliverable. The shared ARB catalog is the interface contract between them.

## References

- Flutter: [Internationalizing Flutter apps](https://docs.flutter.dev/ui/internationalization)
- Weblate: [ARB File format](https://docs.weblate.org/en/latest/formats/arb.html)
- Weblate: [GNU gettext PO format](https://docs.weblate.org/en/latest/formats/gettext.html)
- Unicode ICU: [Formatting Messages](https://unicode-org.github.io/icu/userguide/format_parse/messages/)
- Google: [Application Resource Bundle specification](https://github.com/google/app-resource-bundle/wiki/ApplicationResourceBundleSpecification)
