import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gradus_hello_button/gradus_hello_button.dart';

void main() {
  testWidgets('shows an enabled hello button', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(home: Scaffold(body: GradusHelloButton())),
    );

    final button = find.widgetWithText(ElevatedButton, 'Hello, Gradus!');
    expect(button, findsOneWidget);
    expect(tester.widget<ElevatedButton>(button).enabled, isTrue);

    await tester.tap(button);
  });
}
