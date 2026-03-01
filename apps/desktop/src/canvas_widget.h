#pragma once
#include "camera.h"
#include "doc_store.h"
#include "tools/line_tool.h"
#include "tools/move_tool.h"
#include "tools/rotate_tool.h"
#include "tools/scale_tool.h"
#include "tools/trim_tool.h"
#include "tools/offset_tool.h"
#include "tools/polyline_tool.h"
#include "tools/arc_tool.h"
#include "tools/circle_tool.h"
#include "tools/rect_tool.h"
#include <QWidget>
#include <QJsonObject>

class CanvasWidget : public QWidget {
public:
    explicit CanvasWidget(DocStore* store, QWidget* parent=nullptr);
    void setHighlightedFace(const QJsonObject& face);

protected:
    void paintEvent(QPaintEvent*) override;
    void mousePressEvent(QMouseEvent*) override;
    void mouseMoveEvent(QMouseEvent*) override;
    void mouseReleaseEvent(QMouseEvent*) override;
    void wheelEvent(QWheelEvent*) override;
    void keyPressEvent(QKeyEvent*) override;

private:
    DocStore* store_;
    Camera camera_;
    enum class ActiveTool { Line, Move, Rotate, Scale, Offset, Trim, Rect, Circle, Arc, Polyline };
    ActiveTool activeTool_{ActiveTool::Line};
    LineTool lineTool_;
    MoveTool moveTool_;
    RotateTool rotateTool_;
    ScaleTool scaleTool_;
    OffsetTool offsetTool_;
    TrimTool trimTool_;
    RectTool rectTool_;
    CircleTool circleTool_;
    ArcTool arcTool_;
    PolylineTool polylineTool_;
    ToolBase* currentTool();
    QJsonObject highlightedFace_;
};
