#pragma once
#include <QKeyEvent>
#include <QPainter>
#include <QWheelEvent>

class ToolBase {
public:
    virtual ~ToolBase() = default;
    virtual void onActivate() {}
    virtual void onPointerDown(const QPointF&) {}
    virtual void onPointerMove(const QPointF&) {}
    virtual void onPointerUp(const QPointF&) {}
    virtual void onKeyPress(QKeyEvent*) {}
    virtual void onWheel(QWheelEvent*) {}
    virtual void renderOverlay(QPainter&) {}
};
