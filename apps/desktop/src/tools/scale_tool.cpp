#include "scale_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMessageBox>
#include <cmath>

static QString take_scale(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
static WVec2 scl(const WVec2&p,const WVec2&c,double sx,double sy){ return {c.x + (p.x-c.x)*sx, c.y + (p.y-c.y)*sy}; }

ScaleTool::ScaleTool(DocStore* store, Camera* camera) : store_(store), camera_(camera) {}

void ScaleTool::rebuildCenter() {
    bool first=true; double minx=0,maxx=0,miny=0,maxy=0;
    for (const auto& e : store_->entities()) if (store_->selection().isSelected(e.id) && e.geom.value("type").toString()=="Line") {
        auto a=e.geom.value("a").toObject(); auto b=e.geom.value("b").toObject();
        for (auto p : {QPointF(a.value("x").toDouble(), a.value("y").toDouble()), QPointF(b.value("x").toDouble(), b.value("y").toDouble())}) {
            if(first){minx=maxx=p.x();miny=maxy=p.y();first=false;} else {minx=std::min(minx,p.x());maxx=std::max(maxx,p.x());miny=std::min(miny,p.y());maxy=std::max(maxy,p.y());}
        }
    }
    if (!first) center_ = WVec2{(minx+maxx)*0.5,(miny+maxy)*0.5};
}

void ScaleTool::onPointerDown(const QPointF& p) {
    if (store_->selection().ids().isEmpty()) { QMessageBox::warning(nullptr, "Scale", "EDIT_NO_SELECTION"); return; }
    rebuildCenter(); if(!center_) return;
    start_ = camera_->screenToWorld(p); current_=*start_;
    base_.clear(); for (const auto& e : store_->entities()) if (store_->selection().isSelected(e.id) && e.geom.value("type").toString()=="Line") {
        auto a=e.geom.value("a").toObject(); auto b=e.geom.value("b").toObject();
        base_.push_back({{a.value("x").toDouble(),a.value("y").toDouble()},{b.value("x").toDouble(),b.value("y").toDouble()}});
    }
    refreshPreview(*current_);
    if (!groupActive_) { QByteArray n("Scale"); take_scale(craftcad_history_begin_group(store_->historyHandle(), n.constData())); groupActive_=true; }
}

void ScaleTool::refreshPreview(const WVec2& current) {
    if (!start_ || !center_) return;
    double dx0 = start_->x-center_->x, dy0 = start_->y-center_->y;
    double dx1 = current.x-center_->x, dy1 = current.y-center_->y;
    double sx = std::abs(dx0) > 1e-9 ? dx1/dx0 : 1.0;
    double sy = std::abs(dy0) > 1e-9 ? dy1/dy0 : sx;
    if (QApplication::keyboardModifiers().testFlag(Qt::ShiftModifier)) sy = sx;
    if (auto n = numeric_.value()) sx = sy = *n;
    preview_.clear();
    for (const auto& l : base_) preview_.push_back({scl(l.a,*center_,sx,sy), scl(l.b,*center_,sx,sy)});
}

void ScaleTool::onPointerMove(const QPointF& p) { if(!start_) return; current_=camera_->screenToWorld(p); refreshPreview(*current_); }

bool ScaleTool::commit(double sx, double sy, QString* reason) {
    QByteArray doc = store_->documentJson().toUtf8();
    QJsonArray ids; for (const auto& id : store_->selection().ids()) ids.push_back(id);
    QByteArray sb = QJsonDocument(QJsonObject{{"ids", ids}}).toJson(QJsonDocument::Compact);
    QByteArray tb = QJsonDocument(QJsonObject{{"type","Scale"},{"cx",center_->x},{"cy",center_->y},{"sx",sx},{"sy",sy}}).toJson(QJsonDocument::Compact);
    QByteArray eps = store_->epsPolicyJson().toUtf8();
    QString env = take_scale(craftcad_history_apply_transform_selection(store_->historyHandle(), doc.constData(), sb.constData(), tb.constData(), eps.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) { if(reason) *reason = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()); return false; }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc); return true;
}

void ScaleTool::onPointerUp(const QPointF&) {
    if (!start_ || !current_ || !center_) return;
    double dx0 = start_->x-center_->x, dy0 = start_->y-center_->y;
    double dx1 = current_->x-center_->x, dy1 = current_->y-center_->y;
    double sx = std::abs(dx0) > 1e-9 ? dx1/dx0 : 1.0;
    double sy = std::abs(dy0) > 1e-9 ? dy1/dy0 : sx;
    if (QApplication::keyboardModifiers().testFlag(Qt::ShiftModifier)) sy = sx;
    if (auto n = numeric_.value()) sx = sy = *n;
    if (sx <= 0.0 || sy <= 0.0) { QMessageBox::warning(nullptr,"Scale","EDIT_TRANSFORM_WOULD_DEGENERATE"); return; }
    QString reason; if(!commit(sx,sy,&reason)) QMessageBox::warning(nullptr,"Scale failed",reason);
    take_scale(craftcad_history_end_group(store_->historyHandle())); groupActive_=false; start_.reset(); current_.reset(); preview_.clear(); base_.clear(); numeric_.clear();
}

void ScaleTool::onKeyPress(QKeyEvent* e) {
    if (e->key()==Qt::Key_Escape) { take_scale(craftcad_history_end_group(store_->historyHandle())); groupActive_=false; start_.reset(); current_.reset(); preview_.clear(); base_.clear(); numeric_.clear(); }
    if (e->key()==Qt::Key_Return || e->key()==Qt::Key_Enter) onPointerUp(QPointF());
    numeric_.handleKey(e->key(), e->text());
    if (current_) refreshPreview(*current_);
}

void ScaleTool::renderOverlay(QPainter& p) {
    p.setPen(QPen(QColor(255,160,255), 1.0));
    for (const auto& l : preview_) p.drawLine(camera_->worldToScreen(l.a), camera_->worldToScreen(l.b));
    if (center_) p.drawEllipse(camera_->worldToScreen(*center_), 4, 4);
}
