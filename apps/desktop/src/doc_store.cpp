#include "doc_store.h"
#include "ffi/craftcad_ffi.h"
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>
#include <algorithm>
#include <QSet>

static QString take(char* ptr) {
    if (!ptr) return QString();
    QString s = QString::fromUtf8(ptr);
    craftcad_free_string(ptr);
    return s;
}

DocStore::DocStore() {
    historyHandle_ = craftcad_history_new();
    epsPolicyJson_ = "{\"eq_dist\":1e-6,\"snap_dist\":1e-2,\"intersect_tol\":1e-6,\"area_tol\":1e-6}";
    const QString stateDir = QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation);
    QDir().mkpath(stateDir);
    recoveryPath_ = QDir(stateDir).filePath("recovery_autosave.diycad.json");
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
    ++revision_;
    rebuildCache();
}

void DocStore::rebuildCache() {
    entities_.clear();
    gridBuckets_.clear();
    layerVisible_.clear();
    layerLocked_.clear();

    QJsonObject doc = QJsonDocument::fromJson(documentJson_.toUtf8()).object();
    for (const auto& lv : doc.value("layers").toArray()) {
        auto o = lv.toObject();
        auto id = o.value("id").toString();
        layerVisible_[id] = o.value("visible").toBool(true);
        layerLocked_[id] = o.value("locked").toBool(false);
    }

    int idx = 0;
    for (const auto& ev : doc.value("entities").toArray()) {
        auto e = ev.toObject();
        RenderEntity r;
        r.id = e.value("id").toString();
        r.layerId = e.value("layer_id").toString();
        if (!layerVisible_.value(r.layerId, true) || layerLocked_.value(r.layerId, false)) continue;
        r.geom = e.value("geom").toObject();

        const auto t = r.geom.value("type").toString();
        if (t == "Line") {
            auto a = r.geom.value("a").toObject();
            auto b = r.geom.value("b").toObject();
            double minx = std::min(a.value("x").toDouble(), b.value("x").toDouble());
            double maxx = std::max(a.value("x").toDouble(), b.value("x").toDouble());
            double miny = std::min(a.value("y").toDouble(), b.value("y").toDouble());
            double maxy = std::max(a.value("y").toDouble(), b.value("y").toDouble());
            r.worldAabb = QRectF(QPointF(minx, miny), QPointF(maxx, maxy)).normalized();
        } else if (t == "Circle" || t == "Arc") {
            auto c = r.geom.value("c").toObject();
            const double rr = r.geom.value("r").toDouble();
            r.worldAabb = QRectF(c.value("x").toDouble() - rr, c.value("y").toDouble() - rr, rr * 2.0, rr * 2.0);
        } else if (t == "Polyline") {
            auto pts = r.geom.value("pts").toArray();
            if (!pts.isEmpty()) {
                double minx = 1e300, miny = 1e300, maxx = -1e300, maxy = -1e300;
                for (const auto& pv : pts) {
                    const auto po = pv.toObject();
                    minx = std::min(minx, po.value("x").toDouble());
                    miny = std::min(miny, po.value("y").toDouble());
                    maxx = std::max(maxx, po.value("x").toDouble());
                    maxy = std::max(maxy, po.value("y").toDouble());
                }
                r.worldAabb = QRectF(QPointF(minx, miny), QPointF(maxx, maxy)).normalized();
            }
        }

        entities_.push_back(r);
        const auto& aabb = entities_.back().worldAabb;
        const int cx0 = static_cast<int>(std::floor(aabb.left() / gridCellSize_));
        const int cy0 = static_cast<int>(std::floor(aabb.top() / gridCellSize_));
        const int cx1 = static_cast<int>(std::floor(aabb.right() / gridCellSize_));
        const int cy1 = static_cast<int>(std::floor(aabb.bottom() / gridCellSize_));
        for (int cx = cx0; cx <= cx1; ++cx) {
            for (int cy = cy0; cy <= cy1; ++cy) {
                gridBuckets_[QString::number(cx) + ":" + QString::number(cy)].push_back(idx);
            }
        }
        ++idx;
    }
}

QString DocStore::callDocUpdate(char* ptr, QString* reason, bool* ok) {
    QString env = take(ptr);
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    *ok = root.value("ok").toBool();
    if (!*ok) {
        const QString msg = QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson(QJsonDocument::Compact));
        reasonLogs_.push_back(msg);
        if (reasonLogs_.size() > 200) reasonLogs_.pop_front();
        if (reason) *reason = msg;
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

bool DocStore::writeAtomic(const QString& path, const QByteArray& bytes, QString* reason) const {
    const QString tmp = path + ".tmp";
    QFile f(tmp);
    if (!f.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
        if (reason) *reason = QStringLiteral("failed opening autosave tmp file");
        return false;
    }
    if (f.write(bytes) < 0) {
        if (reason) *reason = QStringLiteral("failed writing autosave tmp file");
        return false;
    }
    f.flush();
    f.close();
    QFile::remove(path);
    if (!QFile::rename(tmp, path)) {
        if (reason) *reason = QStringLiteral("failed renaming autosave tmp file");
        return false;
    }
    return true;
}

bool DocStore::autosaveNow(QString* reason) const {
    return writeAtomic(recoveryPath_, documentJson_.toUtf8(), reason);
}

bool DocStore::hasRecoverySnapshot() const {
    QFileInfo fi(recoveryPath_);
    return fi.exists() && fi.size() > 0;
}

bool DocStore::loadRecoverySnapshot(QString* reason) {
    QFile f(recoveryPath_);
    if (!f.open(QIODevice::ReadOnly)) {
        if (reason) *reason = QStringLiteral("recovery snapshot open failed");
        return false;
    }
    const auto bytes = f.readAll();
    f.close();
    if (bytes.isEmpty()) {
        if (reason) *reason = QStringLiteral("recovery snapshot empty");
        return false;
    }
    setDocumentJson(QString::fromUtf8(bytes));
    return true;
}

bool DocStore::clearRecoverySnapshot(QString* reason) const {
    if (!QFile::exists(recoveryPath_)) return true;
    if (!QFile::remove(recoveryPath_)) {
        if (reason) *reason = QStringLiteral("failed to remove recovery snapshot");
        return false;
    }
    return true;
}

QVector<int> DocStore::querySpatialCandidates(const QPointF& worldPoint, double radiusWorld) const {
    QVector<int> out;
    const int cx0 = static_cast<int>(std::floor((worldPoint.x() - radiusWorld) / gridCellSize_));
    const int cy0 = static_cast<int>(std::floor((worldPoint.y() - radiusWorld) / gridCellSize_));
    const int cx1 = static_cast<int>(std::floor((worldPoint.x() + radiusWorld) / gridCellSize_));
    const int cy1 = static_cast<int>(std::floor((worldPoint.y() + radiusWorld) / gridCellSize_));
    QSet<int> seen;
    for (int cx = cx0; cx <= cx1; ++cx) {
        for (int cy = cy0; cy <= cy1; ++cy) {
            const auto key = QString::number(cx) + ":" + QString::number(cy);
            for (int i : gridBuckets_.value(key)) {
                if (!seen.contains(i)) {
                    seen.insert(i);
                    out.push_back(i);
                }
            }
        }
    }
    return out;
}

QStringList DocStore::latestReasonLogs(int n) const {
    if (n <= 0 || reasonLogs_.isEmpty()) return {};
    const int start = std::max(0, reasonLogs_.size() - n);
    QStringList out;
    for (int i = start; i < reasonLogs_.size(); ++i) out.push_back(reasonLogs_.at(i));
    return out;
}
