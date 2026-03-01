#include "move_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMessageBox>
#include <cmath>

static QString take_move(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

MoveTool::MoveTool(DocStore* store, Camera* camera) : store_(store), camera_(camera) {}

void MoveTool::onPointerDown(const QPointF& p) {
    if (store_->selection().ids().isEmpty()) { QMessageBox::warning(nullptr, "Move", "EDIT_NO_SELECTION"); return; }
    start_ = camera_->screenToWorld(p);
    current_ = *start_;
    base_.clear();
    for (const auto& e : store_->entities()) {
        if (!store_->selection().isSelected(e.id)) continue;
        if (e.geom.value("type").toString() != "Line") continue;
        auto a = e.geom.value("a").toObject(); auto b = e.geom.value("b").toObject();
        base_.push_back({{a.value("x").toDouble(), a.value("y").toDouble()}, {b.value("x").toDouble(), b.value("y").toDouble()}});
    }
    refreshPreview(*current_);
    if (!groupActive_) { QByteArray n("Move"); take_move(craftcad_history_begin_group(store_->historyHandle(), n.constData())); groupActive_=true; }
}

void MoveTool::refreshPreview(const WVec2& current) {
    if (!start_) return;
    double dx = current.x - start_->x;
    double dy = current.y - start_->y;
    if (axis_ == Axis::X) dy = 0.0;
    if (axis_ == Axis::Y) dx = 0.0;
    if (auto n = numeric_.value()) {
        if (axis_ == Axis::X) dx = *n;
        else if (axis_ == Axis::Y) dy = *n;
        else {
            double len = std::sqrt(dx*dx + dy*dy);
            if (len > 0.0) { dx = dx / len * *n; dy = dy / len * *n; }
        }
    }
    preview_.clear();
    for (const auto& l : base_) preview_.push_back({{l.a.x + dx, l.a.y + dy}, {l.b.x + dx, l.b.y + dy}});
}

void MoveTool::onPointerMove(const QPointF& p) {
    if (!start_) return;
    current_ = camera_->screenToWorld(p);
    refreshPreview(*current_);
}

bool MoveTool::commit(double dx, double dy, QString* reason) {
    QByteArray doc = store_->documentJson().toUtf8();
    QJsonArray ids; for (const auto& id : store_->selection().ids()) ids.push_back(id);
    QByteArray sb = QJsonDocument(QJsonObject{{"ids", ids}}).toJson(QJsonDocument::Compact);
    QByteArray tb = QJsonDocument(QJsonObject{{"type","Translate"},{"dx",dx},{"dy",dy}}).toJson(QJsonDocument::Compact);
    QByteArray eps = store_->epsPolicyJson().toUtf8();
    QString env = take_move(craftcad_history_apply_transform_selection(store_->historyHandle(), doc.constData(), sb.constData(), tb.constData(), eps.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) { if(reason) *reason = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()); return false; }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc); return true;
}

void MoveTool::onPointerUp(const QPointF&) {
    if (!start_ || !current_) return;
    double dx = current_->x - start_->x;
    double dy = current_->y - start_->y;
    if (axis_ == Axis::X) dy = 0.0;
    if (axis_ == Axis::Y) dx = 0.0;
    if (auto n = numeric_.value()) {
        if (axis_ == Axis::X) dx = *n;
        else if (axis_ == Axis::Y) dy = *n;
        else {
            double len = std::sqrt(dx*dx + dy*dy);
            if (len > 0.0) { dx = dx / len * *n; dy = dy / len * *n; }
            else { QMessageBox::warning(nullptr,"Move","EDIT_INVALID_NUMERIC"); return; }
        }
    }
    QString reason; if(!commit(dx,dy,&reason)) QMessageBox::warning(nullptr,"Move failed",reason);
    take_move(craftcad_history_end_group(store_->historyHandle())); groupActive_=false; start_.reset(); current_.reset(); numeric_.clear(); preview_.clear(); base_.clear();
}

void MoveTool::onKeyPress(QKeyEvent* e) {
    if (e->key()==Qt::Key_X) axis_ = (axis_==Axis::X?Axis::None:Axis::X);
    if (e->key()==Qt::Key_Y) axis_ = (axis_==Axis::Y?Axis::None:Axis::Y);
    if (e->key()==Qt::Key_Escape) { take_move(craftcad_history_end_group(store_->historyHandle())); groupActive_=false; start_.reset(); current_.reset(); numeric_.clear(); preview_.clear(); base_.clear(); }
    numeric_.handleKey(e->key(), e->text());
    if (current_) refreshPreview(*current_);
}

void MoveTool::renderOverlay(QPainter& p) {
    p.setPen(QPen(QColor(200,255,120), 1.0));
    for (const auto& l : preview_) p.drawLine(camera_->worldToScreen(l.a), camera_->worldToScreen(l.b));
}
