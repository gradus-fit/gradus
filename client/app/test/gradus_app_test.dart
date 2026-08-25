import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gradus_app/gradus_app.dart';

void main() {
  testWidgets('shows the hello button', (tester) async {
    await tester.pumpWidget(const GradusApp());

    final button = find.widgetWithText(ElevatedButton, 'Hello, Gradus!');
    expect(button, findsOneWidget);
    expect(tester.widget<ElevatedButton>(button).enabled, isTrue);

    await tester.tap(button);
  });
}
