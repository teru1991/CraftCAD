#include "canvas_widget.h"
#include "hittest.h"
#include "input/shortcut_map.h"
#include <QKeyEvent>
#include <QPainter>
#include <QMessageBox>

CanvasWidget::CanvasWidget(DocStore* store, QWidget* parent)
    : QWidget(parent), store_(store), lineTool_(store, &camera_), moveTool_(store, &camera_), rotateTool_(store, &camera_), scaleTool_(store, &camera_) {
    setMouseTracking(true);
    setFocusPolicy(Qt::StrongFocus);
}

ToolBase* CanvasWidget::currentTool() {
    switch (activeTool_) {
        case ActiveTool::Line: return &lineTool_;
        case ActiveTool::Move: return &moveTool_;
        case ActiveTool::Rotate: return &rotateTool_;
        case ActiveTool::Scale: return &scaleTool_;
    }
    return &lineTool_;
}

void CanvasWidget::paintEvent(QPaintEvent*) {
    QPainter p(this);
    p.fillRect(rect(), QColor(24,24,24));

    for (const auto& e : store_->entities()) {
        if (e.geom.value("type").toString() != "Line") continue;
        auto a=e.geom.value("a").toObject(); auto b=e.geom.value("b").toObject();
        WVec2 wa{a.value("x").toDouble(), a.value("y").toDouble()};
        WVec2 wb{b.value("x").toDouble(), b.value("y").toDouble()};
        p.setPen(QPen(store_->selection().isSelected(e.id) ? QColor(255,200,0) : QColor(0,220,255), 0));
        p.drawLine(camera_.worldToScreen(wa), camera_.worldToScreen(wb));
    }
    currentTool()->renderOverlay(p);
}

void CanvasWidget::mousePressEvent(QMouseEvent* e) {
    if (e->button()==Qt::LeftButton) {
        auto hit = hitTest(*store_, camera_, e->position(), 8.0);
        if (hit) store_->selection().setSingle(hit->entityId);
        currentTool()->onPointerDown(e->position());
        update();
    }
}

void CanvasWidget::mouseMoveEvent(QMouseEvent* e) { currentTool()->onPointerMove(e->position()); update(); }
void CanvasWidget::mouseReleaseEvent(QMouseEvent* e) { if(e->button()==Qt::LeftButton){ currentTool()->onPointerUp(e->position()); update(); } }

void CanvasWidget::wheelEvent(QWheelEvent* e) {
    double z = e->angleDelta().y() > 0 ? 1.1 : (1.0/1.1);
    camera_.zoom *= z;
    if (camera_.zoom < 0.05) camera_.zoom = 0.05;
    if (camera_.zoom > 100.0) camera_.zoom = 100.0;
    update();
}

void CanvasWidget::keyPressEvent(QKeyEvent* e) {
    QString reason;
    if (isUndo(e)) { if(!store_->undo(&reason)) QMessageBox::warning(this,"Undo failed",reason); update(); return; }
    if (isRedo(e)) { if(!store_->redo(&reason)) QMessageBox::warning(this,"Redo failed",reason); update(); return; }
    if (e->key() == Qt::Key_1) activeTool_ = ActiveTool::Line;
    if (e->key() == Qt::Key_2) activeTool_ = ActiveTool::Move;
    if (e->key() == Qt::Key_3) activeTool_ = ActiveTool::Rotate;
    if (e->key() == Qt::Key_4) activeTool_ = ActiveTool::Scale;
    currentTool()->onKeyPress(e);
    update();
}
