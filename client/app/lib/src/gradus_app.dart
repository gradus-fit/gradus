import 'package:flutter/material.dart';

class GradusApp extends StatelessWidget {
  const GradusApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      themeMode: ThemeMode.dark,
      darkTheme: ThemeData(
        brightness: Brightness.dark,
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFFF5F5F5),
          onPrimary: Color(0xFF121212),
          surface: Color(0xFF121212),
          onSurface: Color(0xFFF5F5F5),
        ),
        scaffoldBackgroundColor: const Color(0xFF121212),
        useMaterial3: true,
      ),
      home: const _ActivityTypeScreen(),
    );
  }
}

class _ActivityTypeScreen extends StatelessWidget {
  const _ActivityTypeScreen();

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;

    return Scaffold(
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(24, 56, 24, 24),
          children: [
            Text('GRADUS', style: textTheme.titleLarge),
            const SizedBox(height: 48),
            const _ActivityTypeButton(
              icon: Icons.directions_run,
              title: 'Running',
            ),
            const SizedBox(height: 16),
            const _ActivityTypeButton(
              icon: Icons.fitness_center,
              title: 'Weightlifting',
            ),
            const SizedBox(height: 16),
            const _ActivityTypeButton(icon: Icons.favorite, title: 'Health'),
          ],
        ),
      ),
    );
  }
}

class _ActivityTypeButton extends StatelessWidget {
  const _ActivityTypeButton({required this.icon, required this.title});

  final IconData icon;
  final String title;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Semantics(
      button: true,
      label: title,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () {},
        child: Stack(
          children: [
            Padding(
              padding: const EdgeInsets.only(top: 28),
              child: Container(
                constraints: const BoxConstraints(minHeight: 112),
                width: double.infinity,
                alignment: Alignment.center,
                padding: const EdgeInsets.fromLTRB(88, 24, 24, 24),
                decoration: BoxDecoration(
                  border: Border.all(color: theme.colorScheme.onSurface),
                  borderRadius: BorderRadius.circular(18),
                ),
                child: Text(title, style: theme.textTheme.titleLarge),
              ),
            ),
            Positioned(
              top: 0,
              left: 20,
              child: IgnorePointer(
                child: Icon(icon, size: 64, color: theme.colorScheme.onSurface),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
