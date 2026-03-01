#include "rotate_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMessageBox>
#include <cmath>

static QString take_rotate(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
static WVec2 rot(const WVec2&p,const WVec2&c,double a){ double cc=std::cos(a), ss=std::sin(a); double x=p.x-c.x,y=p.y-c.y; return {c.x + x*cc - y*ss, c.y + x*ss + y*cc}; }

RotateTool::RotateTool(DocStore* store, Camera* camera) : store_(store), camera_(camera) {}

void RotateTool::rebuildCenter() {
    bool first=true; double minx=0,maxx=0,miny=0,maxy=0;
    for (const auto& e : store_->entities()) {
        if (!store_->selection().isSelected(e.id)) continue;
        if (e.geom.value("type").toString() != "Line") continue;
        auto a = e.geom.value("a").toObject(); auto b = e.geom.value("b").toObject();
        for (auto p : {QPointF(a.value("x").toDouble(), a.value("y").toDouble()), QPointF(b.value("x").toDouble(), b.value("y").toDouble())}) {
            if(first){minx=maxx=p.x();miny=maxy=p.y();first=false;} else {minx=std::min(minx,p.x());maxx=std::max(maxx,p.x());miny=std::min(miny,p.y());maxy=std::max(maxy,p.y());}
        }
    }
    if (!first) center_ = WVec2{(minx+maxx)*0.5,(miny+maxy)*0.5};
}

void RotateTool::onPointerDown(const QPointF& p) {
    if (store_->selection().ids().isEmpty()) { QMessageBox::warning(nullptr, "Rotate", "EDIT_NO_SELECTION"); return; }
    WVec2 w = camera_->screenToWorld(p);
    if (QApplication::keyboardModifiers().testFlag(Qt::AltModifier)) { center_ = w; return; }
    rebuildCenter();
    if (!center_) return;
    start_ = w; current_ = w;
    base_.clear();
    for (const auto& e : store_->entities()) if (store_->selection().isSelected(e.id) && e.geom.value("type").toString()=="Line") {
        auto a=e.geom.value("a").toObject(); auto b=e.geom.value("b").toObject();
        base_.push_back({{a.value("x").toDouble(),a.value("y").toDouble()},{b.value("x").toDouble(),b.value("y").toDouble()}});
    }
    refreshPreview(w);
    if (!groupActive_) { QByteArray n("Rotate"); take_rotate(craftcad_history_begin_group(store_->historyHandle(), n.constData())); groupActive_=true; }
}

void RotateTool::refreshPreview(const WVec2& current) {
    if (!start_ || !center_) return;
    double a0 = std::atan2(start_->y-center_->y,start_->x-center_->x);
    double a1 = std::atan2(current.y-center_->y,current.x-center_->x);
    double angle = a1-a0;
    if (QApplication::keyboardModifiers().testFlag(Qt::ShiftModifier)) {
        double step = 15.0 * M_PI / 180.0;
        angle = std::round(angle / step) * step;
    }
    if (auto n = numeric_.value()) angle = *n * M_PI / 180.0;
    preview_.clear();
    for (const auto& l : base_) preview_.push_back({rot(l.a,*center_,angle), rot(l.b,*center_,angle)});
}

void RotateTool::onPointerMove(const QPointF& p){ if(!start_) return; current_=camera_->screenToWorld(p); refreshPreview(*current_); }

bool RotateTool::commit(double angle, QString* reason) {
    QByteArray doc = store_->documentJson().toUtf8();
    QJsonArray ids; for (const auto& id : store_->selection().ids()) ids.push_back(id);
    QByteArray sb = QJsonDocument(QJsonObject{{"ids", ids}}).toJson(QJsonDocument::Compact);
    QByteArray tb = QJsonDocument(QJsonObject{{"type","Rotate"},{"cx",center_->x},{"cy",center_->y},{"angle_rad",angle}}).toJson(QJsonDocument::Compact);
    QByteArray eps = store_->epsPolicyJson().toUtf8();
    QString env = take_rotate(craftcad_history_apply_transform_selection(store_->historyHandle(), doc.constData(), sb.constData(), tb.constData(), eps.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) { if(reason) *reason = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()); return false; }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc); return true;
}

void RotateTool::onPointerUp(const QPointF&) {
    if (!start_ || !current_ || !center_) return;
    double a0 = std::atan2(start_->y-center_->y,start_->x-center_->x);
    double a1 = std::atan2(current_->y-center_->y,current_->x-center_->x);
    double angle = a1-a0;
    if (QApplication::keyboardModifiers().testFlag(Qt::ShiftModifier)) { double step=15.0*M_PI/180.0; angle=std::round(angle/step)*step; }
    if (auto n = numeric_.value()) angle = *n * M_PI / 180.0;
    QString reason; if(!commit(angle,&reason)) QMessageBox::warning(nullptr,"Rotate failed",reason);
    take_rotate(craftcad_history_end_group(store_->historyHandle())); groupActive_=false; start_.reset(); current_.reset(); preview_.clear(); base_.clear(); numeric_.clear();
}

void RotateTool::onKeyPress(QKeyEvent* e) {
    if (e->key()==Qt::Key_Escape) { take_rotate(craftcad_history_end_group(store_->historyHandle())); groupActive_=false; start_.reset(); current_.reset(); preview_.clear(); base_.clear(); numeric_.clear(); }
    if (e->key()==Qt::Key_Return || e->key()==Qt::Key_Enter) onPointerUp(QPointF());
    numeric_.handleKey(e->key(), e->text());
    if (current_) refreshPreview(*current_);
}

void RotateTool::renderOverlay(QPainter& p) {
    p.setPen(QPen(QColor(150,255,255), 1.0));
    for (const auto& l : preview_) p.drawLine(camera_->worldToScreen(l.a), camera_->worldToScreen(l.b));
    if (center_) p.drawEllipse(camera_->worldToScreen(*center_), 4, 4);
}
