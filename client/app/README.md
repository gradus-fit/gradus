# Gradus app

The Linux Flutter prototype and Client composition root.

The public widget API is available from `package:gradus_app/gradus_app.dart`.
Application implementation remains under `lib/src/`.

## Run locally

Linux builds require `clang`, `ninja-build`, `pkg-config`, and GTK 3 development headers. On Ubuntu:

```bash
sudo apt install clang cmake ninja-build g++ pkg-config libgtk-3-dev
```

Then run:

```bash
cd client
flutter pub get
cd app
flutter run -d linux
```

## Validate

```bash
cd client/app
flutter analyze
flutter test
```

`client/app` contains only application startup and composition. Future features,
storage, and design-system packages will be sibling Client modules.
