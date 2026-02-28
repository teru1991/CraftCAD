#pragma once
#include <QKeyEvent>
#include <QPainter>

class ToolBase {
public:
    virtual ~ToolBase() = default;
    virtual void onActivate() {}
    virtual void onPointerDown(const QPointF&) {}
    virtual void onPointerMove(const QPointF&) {}
    virtual void onPointerUp(const QPointF&) {}
    virtual void onKeyPress(QKeyEvent*) {}
    virtual void renderOverlay(QPainter&) {}
};
