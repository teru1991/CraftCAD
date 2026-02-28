#include "camera.h"

QPointF Camera::worldToScreen(const WVec2& w) const {
    return QPointF((w.x - origin.x) * zoom, -(w.y - origin.y) * zoom);
}

WVec2 Camera::screenToWorld(const QPointF& s) const {
    return WVec2{origin.x + s.x() / zoom, origin.y - s.y() / zoom};
}
