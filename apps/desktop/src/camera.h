#pragma once
#include <QPointF>

struct WVec2 { double x{0.0}; double y{0.0}; };

class Camera {
public:
    WVec2 origin{0.0, 0.0};
    double zoom{1.0};

    QPointF worldToScreen(const WVec2& w) const;
    WVec2 screenToWorld(const QPointF& s) const;
};
