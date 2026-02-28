import 'package:flutter/material.dart';

void main() {
  runApp(const DiyCadViewerApp());
}

class DiyCadViewerApp extends StatelessWidget {
  const DiyCadViewerApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'DIY CAD Viewer',
      home: Scaffold(
        appBar: AppBar(title: const Text('DIY CAD Viewer')),
        body: const Center(
          child: Text(
            'DIY CAD Viewer',
            style: TextStyle(fontSize: 24, fontWeight: FontWeight.w600),
          ),
        ),
      ),
    );
  }
}
