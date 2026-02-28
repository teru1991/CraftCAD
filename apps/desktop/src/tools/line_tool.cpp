#include "line_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>

static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

LineTool::LineTool(DocStore* store, Camera* camera) : store_(store), camera_(camera) {}

WVec2 LineTool::applyConstraint(const WVec2& start, const WVec2& end) const {
    WVec2 out = end;
    if (constraint_ == Constraint::Horizontal) out.y = start.y;
    if (constraint_ == Constraint::Vertical) out.x = start.x;
    return out;
}

void LineTool::onPointerDown(const QPointF& s) {
    WVec2 w = camera_->screenToWorld(s);
    snap_ = computeSnap(*store_, w, std::nullopt);
    start_ = snap_.best ? snap_.best->point : w;
    end_ = start_;
    if (!groupActive_) {
        QByteArray name("LineToolDrag");
        take(craftcad_history_begin_group(store_->historyHandle(), name.constData()));
        groupActive_ = true;
    }
}

void LineTool::onPointerMove(const QPointF& s) {
    if (!start_) return;
    WVec2 w = camera_->screenToWorld(s);
    snap_ = computeSnap(*store_, w, start_);
    WVec2 snapped = snap_.best ? snap_.best->point : w;
    end_ = applyConstraint(*start_, snapped); // precedence: snap first, then constraint
}

bool LineTool::commitLine(const WVec2& start, const WVec2& end, QString* reason) {
    QByteArray doc = store_->documentJson().toUtf8();
    QJsonObject dobj = QJsonDocument::fromJson(doc).object();
    QString layer = dobj.value("layers").toArray().first().toObject().value("id").toString();
    QByteArray lb = layer.toUtf8();
    QByteArray ab = QJsonDocument(QJsonObject{{"x",start.x},{"y",start.y}}).toJson(QJsonDocument::Compact);
    QByteArray bb = QJsonDocument(QJsonObject{{"x",end.x},{"y",end.y}}).toJson(QJsonDocument::Compact);
    QString env = take(craftcad_history_apply_create_line(store_->historyHandle(), doc.constData(), lb.constData(), ab.constData(), bb.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        if (reason) *reason = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson());
        return false;
    }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc);
    return true;
}

void LineTool::onPointerUp(const QPointF&) {
    if (!start_ || !end_) return;
    WVec2 e = *end_;
    if (auto len = numeric_.value()) {
        WVec2 dir{e.x - start_->x, e.y - start_->y};
        double n = std::sqrt(dir.x*dir.x + dir.y*dir.y);
        if (n > 0.0) {
            if (constraint_ == Constraint::Horizontal) dir = {1.0, 0.0};
            else if (constraint_ == Constraint::Vertical) dir = {0.0, 1.0};
            else { dir = {dir.x / n, dir.y / n}; }
            e = {start_->x + dir.x * *len, start_->y + dir.y * *len};
        }
    }
    QString reason;
    if (!commitLine(*start_, e, &reason)) QMessageBox::warning(nullptr, "Line commit failed", reason);
    take(craftcad_history_end_group(store_->historyHandle()));
    groupActive_ = false;
    start_.reset(); end_.reset(); numeric_.clear();
}

void LineTool::onKeyPress(QKeyEvent* e) {
    if (e->key() == Qt::Key_H) constraint_ = (constraint_ == Constraint::Horizontal ? Constraint::None : Constraint::Horizontal);
    if (e->key() == Qt::Key_V) constraint_ = (constraint_ == Constraint::Vertical ? Constraint::None : Constraint::Vertical);
    if (e->key() == Qt::Key_Escape) {
        take(craftcad_history_end_group(store_->historyHandle()));
        groupActive_ = false;
        start_.reset(); end_.reset(); numeric_.clear();
    }
    numeric_.handleKey(e->key(), e->text());
}

void LineTool::renderOverlay(QPainter& p) {
    if (start_ && end_) {
        p.setPen(QPen(QColor(255,210,0), 1.0));
        p.drawLine(camera_->worldToScreen(*start_), camera_->worldToScreen(*end_));
    }
    if (snap_.best) {
        QPointF s = camera_->worldToScreen(snap_.best->point);
        p.setPen(QPen(QColor(255, 80, 80), 1.0));
        p.drawEllipse(s, 4, 4);
    }
}
