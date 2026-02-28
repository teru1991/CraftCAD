#include "snap_engine.h"
#include "ffi/craftcad_ffi.h"
#include <QJsonDocument>

static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

SnapResult computeSnap(const DocStore& store, const WVec2& pointerWorld, const std::optional<WVec2>& lineStart) {
    SnapResult out;
    auto add = [&](WVec2 p, const QString& label, int rank){
        double dx = p.x - pointerWorld.x, dy = p.y - pointerWorld.y;
        out.all.push_back({p,label,rank,std::sqrt(dx*dx+dy*dy)});
    };
    for (const auto& e : store.entities()) {
        auto g = e.geom;
        if (g.value("type").toString()=="Line") {
            auto a=g.value("a").toObject(); auto b=g.value("b").toObject();
            add({a.value("x").toDouble(), a.value("y").toDouble()}, "Endpoint", 0);
            add({b.value("x").toDouble(), b.value("y").toDouble()}, "Endpoint", 0);
            add({(a.value("x").toDouble()+b.value("x").toDouble())*0.5, (a.value("y").toDouble()+b.value("y").toDouble())*0.5}, "Midpoint", 2);
        }
    }
    if (lineStart) {
        QJsonObject active{{"type","Line"},{"a",QJsonObject{{"x",lineStart->x},{"y",lineStart->y}}},{"b",QJsonObject{{"x",pointerWorld.x},{"y",pointerWorld.y}}}};
        QString ajson = QString::fromUtf8(QJsonDocument(active).toJson(QJsonDocument::Compact));
        for (const auto& e : store.entities()) {
            QString bjson = QString::fromUtf8(QJsonDocument(e.geom).toJson(QJsonDocument::Compact));
            QByteArray ab=ajson.toUtf8(), bb=bjson.toUtf8(), eb=store.epsPolicyJson().toUtf8();
            auto root = QJsonDocument::fromJson(take(craftcad_geom_intersect(ab.constData(), bb.constData(), eb.constData())).toUtf8()).object();
            if (!root.value("ok").toBool()) continue;
            for (auto p : root.value("data").toObject().value("points").toArray()) {
                auto po=p.toObject(); add({po.value("x").toDouble(), po.value("y").toDouble()}, "Intersection", 1);
            }
        }
    }
    for (const auto& c: out.all) {
        if (!out.best || c.rank < out.best->rank || (c.rank==out.best->rank && c.dist < out.best->dist)) out.best = c;
    }
    return out;
}
