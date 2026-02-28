#include "hittest.h"
#include "ffi/craftcad_ffi.h"
#include <QJsonDocument>

static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

std::optional<Hit> hitTest(const DocStore& store, const Camera& camera, const QPointF& screenPos, double radiusPx) {
    WVec2 w = camera.screenToWorld(screenPos);
    QJsonObject p{{"x", w.x}, {"y", w.y}};
    QString pjson = QString::fromUtf8(QJsonDocument(p).toJson(QJsonDocument::Compact));
    std::optional<Hit> best;
    for (const auto& e : store.entities()) {
        QRectF ex = e.worldAabb.adjusted(-radiusPx/camera.zoom, -radiusPx/camera.zoom, radiusPx/camera.zoom, radiusPx/camera.zoom);
        if (!ex.contains(QPointF(w.x,w.y))) continue;
        QString gjson = QString::fromUtf8(QJsonDocument(e.geom).toJson(QJsonDocument::Compact));
        QByteArray gb=gjson.toUtf8(), pb=pjson.toUtf8(), eb=store.epsPolicyJson().toUtf8();
        QString env = take(craftcad_geom_project_point(gb.constData(), pb.constData(), eb.constData()));
        auto root = QJsonDocument::fromJson(env.toUtf8()).object();
        if (!root.value("ok").toBool()) continue;
        auto d = root.value("data").toObject();
        double dist = d.value("dist").toDouble(1e9);
        Hit h{e.id, WVec2{d.value("point").toObject().value("x").toDouble(), d.value("point").toObject().value("y").toDouble()}, dist, "Nearest"};
        if (!best || dist < best->dist || (dist==best->dist && h.entityId < best->entityId)) best = h;
    }
    return best;
}
