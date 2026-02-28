#include "doc_store.h"
#include "ffi/craftcad_ffi.h"
#include <QJsonArray>
#include <QJsonDocument>

static QString take(char* ptr) {
    if (!ptr) return QString();
    QString s = QString::fromUtf8(ptr);
    craftcad_free_string(ptr);
    return s;
}

DocStore::DocStore() {
    historyHandle_ = craftcad_history_new();
    epsPolicyJson_ = "{\"eq_dist\":1e-6,\"snap_dist\":1e-2,\"intersect_tol\":1e-6,\"area_tol\":1e-6}";
}

DocStore::~DocStore() { craftcad_history_free(historyHandle_); }

bool DocStore::loadDiycad(const QString& path, QString* errorMsg) {
    QByteArray p = path.toUtf8();
    QString env = take(craftcad_load_diycad_json(p.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        if (errorMsg) *errorMsg = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson());
        return false;
    }
    auto data = root.value("data").toObject();
    setDocumentJson(QString::fromUtf8(QJsonDocument(data.value("document").toObject()).toJson(QJsonDocument::Compact)));
    return true;
}

void DocStore::setDocumentJson(const QString& json) {
    documentJson_ = json;
    rebuildCache();
}

void DocStore::rebuildCache() {
    entities_.clear();
    layerVisible_.clear();
    layerLocked_.clear();
    QJsonObject doc = QJsonDocument::fromJson(documentJson_.toUtf8()).object();
    for (const auto& lv : doc.value("layers").toArray()) {
        auto o = lv.toObject();
        auto id = o.value("id").toString();
        layerVisible_[id] = o.value("visible").toBool(true);
        layerLocked_[id] = o.value("locked").toBool(false);
    }
    for (const auto& ev : doc.value("entities").toArray()) {
        auto e = ev.toObject();
        RenderEntity r;
        r.id = e.value("id").toString();
        r.layerId = e.value("layer_id").toString();
        if (!layerVisible_.value(r.layerId, true) || layerLocked_.value(r.layerId, false)) continue;
        r.geom = e.value("geom").toObject();
        if (r.geom.value("type").toString() == "Line") {
            auto a = r.geom.value("a").toObject();
            auto b = r.geom.value("b").toObject();
            double minx = std::min(a.value("x").toDouble(), b.value("x").toDouble());
            double maxx = std::max(a.value("x").toDouble(), b.value("x").toDouble());
            double miny = std::min(a.value("y").toDouble(), b.value("y").toDouble());
            double maxy = std::max(a.value("y").toDouble(), b.value("y").toDouble());
            r.worldAabb = QRectF(QPointF(minx, miny), QPointF(maxx, maxy)).normalized();
        }
        entities_.push_back(r);
    }
}

QString DocStore::callDocUpdate(char* ptr, QString* reason, bool* ok) {
    QString env = take(ptr);
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    *ok = root.value("ok").toBool();
    if (!*ok) {
        if (reason) *reason = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson());
        return {};
    }
    return QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact));
}

bool DocStore::undo(QString* reason) {
    QByteArray d = documentJson_.toUtf8();
    bool ok = false;
    QString newDoc = callDocUpdate(craftcad_history_undo(historyHandle_, d.constData()), reason, &ok);
    if (ok) setDocumentJson(newDoc);
    return ok;
}

bool DocStore::redo(QString* reason) {
    QByteArray d = documentJson_.toUtf8();
    bool ok = false;
    QString newDoc = callDocUpdate(craftcad_history_redo(historyHandle_, d.constData()), reason, &ok);
    if (ok) setDocumentJson(newDoc);
    return ok;
}
