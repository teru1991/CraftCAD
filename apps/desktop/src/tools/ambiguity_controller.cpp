#include "ambiguity_controller.h"
void AmbiguityController::onWheel(QWheelEvent* e) { if (candidates_.isEmpty()) return; onTab(e->angleDelta().y()>=0?1:-1); }
void AmbiguityController::onTab(int delta){ if(candidates_.isEmpty()) return; index_=(index_+delta)%candidates_.size(); if(index_<0) index_+=candidates_.size(); }
void AmbiguityController::render(QPainter& p, const Camera& camera) const {
    if (candidates_.isEmpty()) return;
    for (int i=0;i<candidates_.size();++i){ auto o=candidates_[i].toObject(); QPointF s=camera.worldToScreen({o.value("x").toDouble(),o.value("y").toDouble()}); p.setPen(QPen(i==index_?QColor(255,255,0):QColor(255,120,120),1)); p.drawEllipse(s, i==index_?6:4, i==index_?6:4); }
}
