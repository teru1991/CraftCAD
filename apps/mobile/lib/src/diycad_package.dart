import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';

import 'models.dart';

class DiycadPackage {
  DiycadPackage({
    required this.document,
    required this.rawFiles,
    required this.availableExports,
  });

  final DiycadDocument document;
  final Map<String, Uint8List> rawFiles;
  final Map<String, Uint8List> availableExports;

  static DiycadPackage fromBytes(Uint8List bytes) {
    final archive = ZipDecoder().decodeBytes(bytes, verify: true);
    final files = <String, Uint8List>{};
    for (final f in archive.files.where((f) => f.isFile)) {
      files[f.name] = Uint8List.fromList((f.content as List).cast<int>());
    }

    final docBytes = files['data/document.json'];
    if (docBytes == null) {
      throw const FormatException('Missing data/document.json in .diycad package');
    }
    final doc = DiycadDocument.fromJsonString(utf8.decode(docBytes));

    final exports = <String, Uint8List>{};
    for (final e in files.entries) {
      final n = e.key.toLowerCase();
      if (n.endsWith('.pdf') || n.endsWith('.svg')) {
        exports[e.key] = e.value;
      }
    }

    return DiycadPackage(document: doc, rawFiles: files, availableExports: exports);
  }
}
