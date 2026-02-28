#pragma once
#include "camera.h"
#include "doc_store.h"
#include "tools/line_tool.h"
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
    LineTool lineTool_;
};
