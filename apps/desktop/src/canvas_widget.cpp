#include "canvas_widget.h"
#include "hittest.h"
#include "input/shortcut_map.h"
#include <QKeyEvent>
#include <QPainter>
#include <QMessageBox>

CanvasWidget::CanvasWidget(DocStore* store, QWidget* parent)
    : QWidget(parent), store_(store), lineTool_(store, &camera_) {
    setMouseTracking(true);
    setFocusPolicy(Qt::StrongFocus);
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
    lineTool_.renderOverlay(p);
}

void CanvasWidget::mousePressEvent(QMouseEvent* e) {
    if (e->button()==Qt::LeftButton) {
        auto hit = hitTest(*store_, camera_, e->position(), 8.0);
        if (hit) store_->selection().setSingle(hit->entityId);
        lineTool_.onPointerDown(e->position());
        update();
    }
}

void CanvasWidget::mouseMoveEvent(QMouseEvent* e) { lineTool_.onPointerMove(e->position()); update(); }
void CanvasWidget::mouseReleaseEvent(QMouseEvent* e) { if(e->button()==Qt::LeftButton){ lineTool_.onPointerUp(e->position()); update(); } }

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
    lineTool_.onKeyPress(e);
    update();
}
