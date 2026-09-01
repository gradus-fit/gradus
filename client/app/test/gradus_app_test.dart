import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gradus_app/gradus_app.dart';

void main() {
  testWidgets('shows activity choices in dark mode', (tester) async {
    final semantics = tester.ensureSemantics();
    await tester.pumpWidget(const GradusApp());

    final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
    expect(app.themeMode, ThemeMode.dark);
    expect(app.darkTheme!.brightness, Brightness.dark);
    expect(find.text('GRADUS'), findsOneWidget);
    expect(find.text('Running'), findsOneWidget);
    expect(find.text('Weightlifting'), findsOneWidget);
    expect(find.text('Health'), findsOneWidget);
    expect(find.byIcon(Icons.directions_run), findsOneWidget);
    expect(find.byIcon(Icons.fitness_center), findsOneWidget);
    expect(find.byIcon(Icons.favorite), findsOneWidget);
    expect(find.bySemanticsLabel(RegExp('Running')), findsOneWidget);
    expect(find.bySemanticsLabel(RegExp('Weightlifting')), findsOneWidget);
    expect(find.bySemanticsLabel(RegExp('Health')), findsOneWidget);
    semantics.dispose();
  });
}
