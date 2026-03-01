import 'dart:io';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

import 'src/annotation_store.dart';
import 'src/diycad_package.dart';
import 'src/viewer_painter.dart';

void main() {
  runApp(const DiyCadViewerApp());
}

class DiyCadViewerApp extends StatelessWidget {
  const DiyCadViewerApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'CraftCAD Mobile Viewer',
      theme: ThemeData(useMaterial3: true),
      home: const ViewerScreen(),
    );
  }
}

class ViewerScreen extends StatefulWidget {
  const ViewerScreen({super.key});

  @override
  State<ViewerScreen> createState() => _ViewerScreenState();
}

class _ViewerScreenState extends State<ViewerScreen> {
  DiycadPackage? _package;
  String _status = 'Open a .diycad package to begin.';
  final List<Offset> _annotationPoints = [];

  Future<void> _openDiycad() async {
    final picked = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['diycad'],
      withData: true,
    );
    final bytes = picked?.files.single.bytes;
    if (bytes == null) return;

    try {
      final parsed = DiycadPackage.fromBytes(bytes);
      setState(() {
        _package = parsed;
        _status =
            'Loaded doc=${parsed.document.id}, entities=${parsed.document.entities.length}, nesting placements=${parsed.document.placements.length}';
      });
    } catch (e) {
      setState(() => _status = 'Failed to open .diycad: $e');
    }
  }

  Future<void> _saveAnnotationsAsNote() async {
    final doc = _package?.document;
    if (doc == null) return;
    final now = DateTime.now().toUtc().toIso8601String();
    final package = AnnotationPackage(
      docId: doc.id,
      annotations: [
        Annotation(
          id: 'ann-${now.hashCode}',
          page: 'sheet-0',
          points: _annotationPoints
              .map((o) => {'x': o.dx, 'y': o.dy})
              .toList(growable: false),
          text: 'mobile note',
          color: '#E11D48',
          createdAt: now,
        )
      ],
    );
    final bytes = package.toZipBytes();
    final dir = await getApplicationDocumentsDirectory();
    final path = p.join(dir.path, '${doc.id}.diycad-note');
    await File(path).writeAsBytes(bytes, flush: true);
    setState(() => _status = 'Saved annotations to $path');
  }

  Future<void> _shareExport() async {
    final pkg = _package;
    if (pkg == null) return;
    if (pkg.availableExports.isEmpty) {
      setState(() {
        _status =
            'No embedded PDF/SVG exports found in package. Generate exports on desktop first.';
      });
      return;
    }

    final entry = pkg.availableExports.entries.first;
    final tempDir = await getTemporaryDirectory();
    final file = File(p.join(tempDir.path, p.basename(entry.key)));
    await file.writeAsBytes(entry.value, flush: true);
    await Share.shareXFiles([XFile(file.path)], text: 'CraftCAD export share');
    setState(() => _status = 'Shared ${entry.key}');
  }

  @override
  Widget build(BuildContext context) {
    final doc = _package?.document;
    return Scaffold(
      appBar: AppBar(
        title: const Text('CraftCAD Mobile Viewer'),
        actions: [
          IconButton(onPressed: _openDiycad, icon: const Icon(Icons.folder_open)),
          IconButton(
            onPressed: _saveAnnotationsAsNote,
            icon: const Icon(Icons.edit_note),
          ),
          IconButton(onPressed: _shareExport, icon: const Icon(Icons.share)),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Text(_status, style: const TextStyle(fontSize: 12)),
          ),
          Expanded(
            child: Container(
              margin: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                border: Border.all(color: Colors.black12),
                borderRadius: BorderRadius.circular(12),
                color: Colors.white,
              ),
              child: doc == null
                  ? const Center(child: Text('No document loaded'))
                  : GestureDetector(
                      onTapDown: (d) {
                        setState(() => _annotationPoints.add(d.localPosition));
                      },
                      child: CustomPaint(
                        painter: ViewerPainter(
                          doc: doc,
                          annotations: _annotationPoints,
                        ),
                        child: const SizedBox.expand(),
                      ),
                    ),
            ),
          ),
        ],
      ),
      floatingActionButton: doc == null
          ? null
          : FloatingActionButton.extended(
              onPressed: () => setState(_annotationPoints.clear),
              label: const Text('Clear Notes'),
              icon: const Icon(Icons.delete_sweep),
            ),
    );
  }
}
