#include "offset_tool.h"
#include "../ffi/craftcad_ffi.h"
#include "../hittest.h"
#include <QJsonDocument>
#include <QJsonObject>
#include <QMessageBox>

static QString take_offset(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

OffsetTool::OffsetTool(DocStore* store, Camera* camera) : store_(store), camera_(camera) {}

void OffsetTool::onPointerDown(const QPointF& p) {
    auto hit = hitTest(*store_, *camera_, p, 8.0);
    if (!hit) { QMessageBox::warning(nullptr, "Offset", "EDIT_NO_SELECTION"); return; }
    targetId_ = hit->entityId;
    store_->selection().setSingle(targetId_);
    updatePreview();
}

void OffsetTool::onPointerMove(const QPointF&) { updatePreview(); }

void OffsetTool::updatePreview() {
    if (targetId_.isEmpty()) return;
    auto dist = numeric_.value().value_or(10.0);
    previewDist_ = dist;
    for (const auto& e : store_->entities()) {
        if (e.id != targetId_) continue;
        if (e.geom.value("type").toString() == "Line") {
            auto a = e.geom.value("a").toObject();
            auto b = e.geom.value("b").toObject();
            double dx = b.value("x").toDouble() - a.value("x").toDouble();
            double dy = b.value("y").toDouble() - a.value("y").toDouble();
            double len = std::sqrt(dx*dx + dy*dy);
            if (len <= 1e-9) return;
            double nx = -dy / len;
            double ny = dx / len;
            previewGeom_ = QJsonObject{{"type","Line"},
                {"a",QJsonObject{{"x",a.value("x").toDouble()+nx*dist},{"y",a.value("y").toDouble()+ny*dist}}},
                {"b",QJsonObject{{"x",b.value("x").toDouble()+nx*dist},{"y",b.value("y").toDouble()+ny*dist}}}};
        }
    }
}

bool OffsetTool::commit(QString* reason) {
    if (targetId_.isEmpty()) return false;
    QByteArray doc = store_->documentJson().toUtf8();
    QByteArray id = targetId_.toUtf8();
    double d = previewDist_.value_or(10.0);
    QByteArray eps = store_->epsPolicyJson().toUtf8();
    QString env = take_offset(craftcad_history_apply_offset_entity(store_->historyHandle(), doc.constData(), id.constData(), d, eps.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        if (reason) *reason = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson());
        return false;
    }
    QString newDoc = QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
    store_->setDocumentJson(newDoc);
    return true;
}

void OffsetTool::onPointerUp(const QPointF&) {
    QString reason;
    if (!commit(&reason)) QMessageBox::warning(nullptr, "Offset failed", reason);
    previewGeom_ = QJsonObject{};
}

void OffsetTool::onKeyPress(QKeyEvent* e) {
    if (e->key() == Qt::Key_Escape) { targetId_.clear(); previewGeom_ = QJsonObject{}; numeric_.clear(); return; }
    numeric_.handleKey(e->key(), e->text());
    if (e->key() == Qt::Key_Return || e->key() == Qt::Key_Enter) {
        QString reason;
        if (!commit(&reason)) QMessageBox::warning(nullptr, "Offset failed", reason);
    }
    updatePreview();
}

void OffsetTool::renderOverlay(QPainter& p) {
    if (previewGeom_.isEmpty()) return;
    if (previewGeom_.value("type").toString() == "Line") {
        auto a = previewGeom_.value("a").toObject();
        auto b = previewGeom_.value("b").toObject();
        p.setPen(QPen(QColor(120,255,120), 1.0));
        p.drawLine(camera_->worldToScreen({a.value("x").toDouble(), a.value("y").toDouble()}),
                   camera_->worldToScreen({b.value("x").toDouble(), b.value("y").toDouble()}));
    }
}
