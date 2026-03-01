import 'dart:convert';

class Vec2 {
  const Vec2(this.x, this.y);
  final double x;
  final double y;

  factory Vec2.fromJson(Map<String, dynamic> json) =>
      Vec2((json['x'] as num).toDouble(), (json['y'] as num).toDouble());

  Map<String, dynamic> toJson() => {'x': x, 'y': y};
}

sealed class Geom {
  const Geom();
  factory Geom.fromJson(Map<String, dynamic> json) {
    switch (json['type']) {
      case 'Line':
        return LineGeom(Vec2.fromJson(json['a']), Vec2.fromJson(json['b']));
      case 'Circle':
        return CircleGeom(Vec2.fromJson(json['c']), (json['r'] as num).toDouble());
      case 'Arc':
        return ArcGeom(
          c: Vec2.fromJson(json['c']),
          r: (json['r'] as num).toDouble(),
          startAngle: (json['start_angle'] as num).toDouble(),
          endAngle: (json['end_angle'] as num).toDouble(),
          ccw: json['ccw'] as bool,
        );
      case 'Polyline':
        return PolylineGeom(
          pts: (json['pts'] as List<dynamic>)
              .map((e) => Vec2.fromJson(e as Map<String, dynamic>))
              .toList(growable: false),
          closed: json['closed'] as bool,
        );
      default:
        throw FormatException('Unsupported geom type: ${json['type']}');
    }
  }
}

class LineGeom extends Geom {
  const LineGeom(this.a, this.b);
  final Vec2 a;
  final Vec2 b;
}

class CircleGeom extends Geom {
  const CircleGeom(this.c, this.r);
  final Vec2 c;
  final double r;
}

class ArcGeom extends Geom {
  const ArcGeom({
    required this.c,
    required this.r,
    required this.startAngle,
    required this.endAngle,
    required this.ccw,
  });

  final Vec2 c;
  final double r;
  final double startAngle;
  final double endAngle;
  final bool ccw;
}

class PolylineGeom extends Geom {
  const PolylineGeom({required this.pts, required this.closed});
  final List<Vec2> pts;
  final bool closed;
}

class Entity {
  const Entity({required this.id, required this.layerId, required this.geom});
  final String id;
  final String layerId;
  final Geom geom;

  factory Entity.fromJson(Map<String, dynamic> json) => Entity(
        id: json['id'] as String,
        layerId: json['layer_id'] as String,
        geom: Geom.fromJson(json['geom'] as Map<String, dynamic>),
      );
}

class NestPlacement {
  const NestPlacement({
    required this.partId,
    required this.sheetInstanceIndex,
    required this.x,
    required this.y,
    required this.rotationDeg,
    required this.bbox,
  });
  final String partId;
  final int sheetInstanceIndex;
  final double x;
  final double y;
  final double rotationDeg;
  final Map<String, dynamic>? bbox;

  factory NestPlacement.fromJson(Map<String, dynamic> json) => NestPlacement(
        partId: json['part_id'] as String,
        sheetInstanceIndex: json['sheet_instance_index'] as int,
        x: (json['x'] as num).toDouble(),
        y: (json['y'] as num).toDouble(),
        rotationDeg: (json['rotation_deg'] as num).toDouble(),
        bbox: json['bbox'] as Map<String, dynamic>?,
      );
}

class DiycadDocument {
  const DiycadDocument({
    required this.id,
    required this.units,
    required this.entities,
    required this.placements,
    required this.raw,
  });

  final String id;
  final String units;
  final List<Entity> entities;
  final List<NestPlacement> placements;
  final Map<String, dynamic> raw;

  factory DiycadDocument.fromJsonString(String src) {
    final json = jsonDecode(src) as Map<String, dynamic>;
    final entities = ((json['entities'] as List<dynamic>? ?? const [])
            .map((e) => Entity.fromJson(e as Map<String, dynamic>))
            .toList()
          ..sort((a, b) => a.id.compareTo(b.id)))
        .toList(growable: false);

    final placements = <NestPlacement>[];
    for (final job in (json['jobs'] as List<dynamic>? ?? const [])) {
      final j = job as Map<String, dynamic>;
      final result = j['result'];
      if (result is Map<String, dynamic>) {
        final p = result['placements'];
        if (p is List<dynamic>) {
          placements.addAll(p
              .whereType<Map<String, dynamic>>()
              .map(NestPlacement.fromJson)
              .toList(growable: false));
        }
      }
    }
    placements.sort((a, b) {
      final s = a.sheetInstanceIndex.compareTo(b.sheetInstanceIndex);
      if (s != 0) return s;
      return a.partId.compareTo(b.partId);
    });

    return DiycadDocument(
      id: json['id'] as String? ?? 'unknown',
      units: json['units'] as String? ?? 'mm',
      entities: entities,
      placements: placements,
      raw: json,
    );
  }
}
