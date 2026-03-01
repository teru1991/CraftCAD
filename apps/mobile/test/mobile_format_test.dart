import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:diycad_mobile/src/annotation_store.dart';
import 'package:diycad_mobile/src/diycad_package.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('diycad package parsing is deterministic and supports basic geoms', () {
    final docJson = {
      'id': 'doc-1',
      'units': 'mm',
      'entities': [
        {
          'id': 'b',
          'layer_id': 'l',
          'geom': {
            'type': 'Circle',
            'c': {'x': 3, 'y': 4},
            'r': 2,
          },
          'style': {},
          'tags': [],
          'meta': {},
        },
        {
          'id': 'a',
          'layer_id': 'l',
          'geom': {
            'type': 'Line',
            'a': {'x': 0, 'y': 0},
            'b': {'x': 10, 'y': 0},
          },
          'style': {},
          'tags': [],
          'meta': {},
        },
      ],
      'jobs': [
        {
          'id': 'j',
          'sheet_defs': [],
          'parts_ref': [],
          'constraints': {
            'global_margin': 0,
            'global_kerf': 0,
            'allow_rotate_default': true,
            'no_go_zones': [],
            'grain_policy': 'Ignore'
          },
          'objective': {'w_utilization': 1, 'w_sheet_count': 1, 'w_cut_count': 1},
          'seed': 1,
          'result': {
            'placements': [
              {
                'part_id': 'p1',
                'sheet_instance_index': 0,
                'x': 1,
                'y': 2,
                'rotation_deg': 0,
                'bbox': {
                  'min': {'x': 1, 'y': 2},
                  'max': {'x': 3, 'y': 4}
                }
              }
            ]
          },
          'trace': null,
        }
      ]
    };

    final archive = Archive()
      ..addFile(ArchiveFile(
        'data/document.json',
        0,
        utf8.encode(jsonEncode(docJson)),
      ));
    final bytes = Uint8List.fromList(ZipEncoder().encode(archive)!);

    final parsed1 = DiycadPackage.fromBytes(bytes);
    final parsed2 = DiycadPackage.fromBytes(bytes);

    expect(parsed1.document.entities.first.id, 'a');
    expect(parsed1.document.placements.length, 1);
    expect(
      jsonEncode(parsed1.document.raw),
      jsonEncode(parsed2.document.raw),
      reason: 'same input must parse identically',
    );
  });

  test('annotation package roundtrip', () {
    final pkg = AnnotationPackage(docId: 'doc-1', annotations: const [
      Annotation(
        id: 'ann-1',
        page: 'sheet-0',
        points: [
          {'x': 1.0, 'y': 2.0},
          {'x': 3.0, 'y': 4.0}
        ],
        text: 'note',
        color: '#E11D48',
        createdAt: '2026-03-01T00:00:00Z',
      )
    ]);

    final bytes = pkg.toZipBytes();
    final decoded = AnnotationPackage.fromZipBytes(bytes);
    expect(decoded.docId, 'doc-1');
    expect(decoded.annotations.length, 1);
    expect(decoded.annotations.first.points.length, 2);
  });
}
