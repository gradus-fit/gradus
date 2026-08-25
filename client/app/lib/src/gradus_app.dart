import 'package:flutter/material.dart';
import 'package:gradus_hello_button/gradus_hello_button.dart';

class GradusApp extends StatelessWidget {
  const GradusApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        body: Center(child: GradusHelloButton()),
      ),
    );
  }
}
