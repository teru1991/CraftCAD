#include "overlay_renderer.h"

void renderOverlay(QPainter& p, const Camera& cam, const SnapResult& snap, const std::optional<QPair<WVec2,WVec2>>& previewLine) {
    if (previewLine) {
        p.setPen(QPen(QColor(255, 210, 0), 1.0));
        QPointF a = cam.worldToScreen(previewLine->first);
        QPointF b = cam.worldToScreen(previewLine->second);
        p.drawLine(a,b);
    }
    if (snap.best) {
        QPointF s = cam.worldToScreen(snap.best->point);
        p.setPen(QPen(QColor(255, 80, 80), 1.0));
        p.drawLine(s + QPointF(-6,0), s + QPointF(6,0));
        p.drawLine(s + QPointF(0,-6), s + QPointF(0,6));
        p.drawText(s + QPointF(8,-8), snap.best->label);
    }
}
