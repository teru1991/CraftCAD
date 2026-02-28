#pragma once
#include "camera.h"
#include "doc_store.h"
#include "tools/line_tool.h"
#include "tools/move_tool.h"
#include "tools/rotate_tool.h"
#include "tools/scale_tool.h"
#include <QWidget>

class CanvasWidget : public QWidget {
public:
    explicit CanvasWidget(DocStore* store, QWidget* parent=nullptr);

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
    enum class ActiveTool { Line, Move, Rotate, Scale };
    ActiveTool activeTool_{ActiveTool::Line};
    LineTool lineTool_;
    MoveTool moveTool_;
    RotateTool rotateTool_;
    ScaleTool scaleTool_;
    ToolBase* currentTool();
};
