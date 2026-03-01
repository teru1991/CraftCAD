import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';

class Annotation {
  const Annotation({
    required this.id,
    required this.page,
    required this.points,
    required this.text,
    required this.createdAt,
    this.color,
  });

  final String id;
  final String page;
  final List<Map<String, double>> points;
  final String text;
  final String createdAt;
  final String? color;

  Map<String, dynamic> toJson() => {
        'id': id,
        'page': page,
        'points': points,
        'text': text,
        if (color != null) 'color': color,
        'created_at': createdAt,
      };

  factory Annotation.fromJson(Map<String, dynamic> json) => Annotation(
        id: json['id'] as String,
        page: json['page'] as String,
        points: (json['points'] as List<dynamic>)
            .map((e) => {
                  'x': (e['x'] as num).toDouble(),
                  'y': (e['y'] as num).toDouble(),
                })
            .toList(growable: false),
        text: json['text'] as String,
        color: json['color'] as String?,
        createdAt: json['created_at'] as String,
      );
}

class AnnotationPackage {
  const AnnotationPackage({required this.docId, required this.annotations});

  final String docId;
  final List<Annotation> annotations;

  Uint8List toZipBytes() {
    final archive = Archive();
    final manifest = jsonEncode({
      'kind': 'diycad-note',
      'version': 1,
      'doc_id': docId,
    });
    final payload = jsonEncode({
      'doc_id': docId,
      'annotations': annotations.map((a) => a.toJson()).toList(),
    });

    archive.addFile(ArchiveFile('manifest.json', manifest.length, utf8.encode(manifest)));
    archive.addFile(
      ArchiveFile('data/annotations.json', payload.length, utf8.encode(payload)),
    );
    return Uint8List.fromList(ZipEncoder().encode(archive)!);
  }

  factory AnnotationPackage.fromZipBytes(Uint8List bytes) {
    final archive = ZipDecoder().decodeBytes(bytes, verify: true);
    final payloadFile = archive.findFile('data/annotations.json');
    if (payloadFile == null) {
      throw const FormatException('Missing data/annotations.json');
    }
    final payload = utf8.decode(payloadFile.content as List<int>);
    final json = jsonDecode(payload) as Map<String, dynamic>;
    return AnnotationPackage(
      docId: json['doc_id'] as String,
      annotations: (json['annotations'] as List<dynamic>)
          .map((e) => Annotation.fromJson(e as Map<String, dynamic>))
          .toList(growable: false),
    );
  }
}
