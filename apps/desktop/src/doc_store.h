#pragma once
#include "selection_state.h"
#include <QHash>
#include <QJsonObject>
#include <QRectF>
#include <QString>
#include <QVector>

struct RenderEntity {
    QString id;
    QString layerId;
    QJsonObject geom;
    QRectF worldAabb;
};

class DocStore {
public:
    DocStore();
    ~DocStore();

    bool loadDiycad(const QString& path, QString* errorMsg);
    void setDocumentJson(const QString& json);
    const QString& documentJson() const { return documentJson_; }

    const QVector<RenderEntity>& entities() const { return entities_; }
    const QString& epsPolicyJson() const { return epsPolicyJson_; }
    SelectionState& selection() { return selection_; }

    bool undo(QString* reason);
    bool redo(QString* reason);

    uint64_t historyHandle() const { return historyHandle_; }

    // crash-safe autosave/recovery
    bool autosaveNow(QString* reason = nullptr) const;
    bool hasRecoverySnapshot() const;
    bool loadRecoverySnapshot(QString* reason);
    bool clearRecoverySnapshot(QString* reason = nullptr) const;
    QString recoveryPath() const { return recoveryPath_; }

    // spatial index (uniform grid)
    QVector<int> querySpatialCandidates(const QPointF& worldPoint, double radiusWorld) const;
    quint64 revision() const { return revision_; }
    QStringList latestReasonLogs(int n) const;

private:
    void rebuildCache();
    QString callDocUpdate(char* ptr, QString* reason, bool* ok);
    bool writeAtomic(const QString& path, const QByteArray& bytes, QString* reason) const;

    uint64_t historyHandle_{0};
    QString documentJson_;
    QVector<RenderEntity> entities_;
    QString epsPolicyJson_;
    SelectionState selection_;
    QHash<QString, bool> layerVisible_;
    QHash<QString, bool> layerLocked_;

    // autosave location
    QString recoveryPath_;

    // spatial index data
    double gridCellSize_{64.0};
    QHash<QString, QVector<int>> gridBuckets_;
    quint64 revision_{0};
    QStringList reasonLogs_;
};
