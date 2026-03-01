import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'models.dart';

class ViewerPainter extends CustomPainter {
  ViewerPainter({required this.doc, required this.annotations});

  final DiycadDocument doc;
  final List<Offset> annotations;

  @override
  void paint(Canvas canvas, Size size) {
    final bbox = _computeBounds(doc);
    final scaleX = size.width / (bbox.width == 0 ? 1 : bbox.width);
    final scaleY = size.height / (bbox.height == 0 ? 1 : bbox.height);
    final scale = math.min(scaleX, scaleY) * 0.9;
    final tx = size.width / 2 - (bbox.center.dx) * scale;
    final ty = size.height / 2 + (bbox.center.dy) * scale;

    Offset map(Vec2 v) => Offset(v.x * scale + tx, -v.y * scale + ty);

    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..color = const Color(0xFF1D2939);

    for (final e in doc.entities) {
      final g = e.geom;
      if (g is LineGeom) {
        canvas.drawLine(map(g.a), map(g.b), stroke);
      } else if (g is PolylineGeom) {
        if (g.pts.isEmpty) continue;
        final p = Path()..moveTo(map(g.pts.first).dx, map(g.pts.first).dy);
        for (final pt in g.pts.skip(1)) {
          final m = map(pt);
          p.lineTo(m.dx, m.dy);
        }
        if (g.closed) p.close();
        canvas.drawPath(p, stroke);
      } else if (g is CircleGeom) {
        canvas.drawCircle(map(g.c), g.r * scale, stroke);
      } else if (g is ArcGeom) {
        final c = map(g.c);
        final rect = Rect.fromCircle(center: c, radius: g.r * scale);
        final sweep = g.ccw
            ? _norm(g.endAngle - g.startAngle)
            : -_norm(g.startAngle - g.endAngle);
        canvas.drawArc(rect, -g.startAngle, -sweep, false, stroke);
      }
    }

    final nestPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = const Color(0xFF7C3AED);
    for (final p in doc.placements) {
      final b = p.bbox;
      if (b == null || !b.containsKey('min') || !b.containsKey('max')) continue;
      final min = Vec2.fromJson(b['min'] as Map<String, dynamic>);
      final max = Vec2.fromJson(b['max'] as Map<String, dynamic>);
      final r = Rect.fromPoints(map(min), map(max));
      canvas.drawRect(r, nestPaint);
    }

    final annPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..color = const Color(0xFFE11D48);
    for (final a in annotations) {
      canvas.drawCircle(a, 4, annPaint);
    }
  }

  @override
  bool shouldRepaint(covariant ViewerPainter oldDelegate) =>
      oldDelegate.doc != doc || oldDelegate.annotations != annotations;

  double _norm(double a) {
    while (a < 0) {
      a += 2 * math.pi;
    }
    while (a >= 2 * math.pi) {
      a -= 2 * math.pi;
    }
    return a;
  }
}

Rect _computeBounds(DiycadDocument doc) {
  double minX = double.infinity;
  double minY = double.infinity;
  double maxX = double.negativeInfinity;
  double maxY = double.negativeInfinity;

  void hit(Vec2 p) {
    minX = math.min(minX, p.x);
    minY = math.min(minY, p.y);
    maxX = math.max(maxX, p.x);
    maxY = math.max(maxY, p.y);
  }

  for (final e in doc.entities) {
    final g = e.geom;
    if (g is LineGeom) {
      hit(g.a);
      hit(g.b);
    } else if (g is PolylineGeom) {
      for (final p in g.pts) hit(p);
    } else if (g is CircleGeom) {
      hit(Vec2(g.c.x - g.r, g.c.y - g.r));
      hit(Vec2(g.c.x + g.r, g.c.y + g.r));
    } else if (g is ArcGeom) {
      hit(Vec2(g.c.x - g.r, g.c.y - g.r));
      hit(Vec2(g.c.x + g.r, g.c.y + g.r));
    }
  }

  if (!minX.isFinite) {
    return const Rect.fromLTWH(-50, -50, 100, 100);
  }
  return Rect.fromLTRB(minX, minY, maxX, maxY);
}
