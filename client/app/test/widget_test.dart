import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gradus_app/gradus_app.dart';

void main() {
  testWidgets('activity choices and icons can be tapped', (tester) async {
    await tester.pumpWidget(const GradusApp());

    await tester.tap(find.text('Running'));
    await tester.tapAt(tester.getCenter(find.byIcon(Icons.directions_run)));
    expect(tester.takeException(), isNull);
  });

  testWidgets('activity choices support large text on a narrow viewport', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(320, 500);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(
      const MediaQuery(
        data: MediaQueryData(textScaler: TextScaler.linear(3)),
        child: GradusApp(),
      ),
    );

    await tester.scrollUntilVisible(find.text('Weightlifting'), 100);
    expect(find.text('Weightlifting'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
