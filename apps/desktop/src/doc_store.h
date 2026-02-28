#pragma once
#include "selection_state.h"
#include <QHash>
#include <QJsonObject>
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

private:
    void rebuildCache();
    QString callDocUpdate(char* ptr, QString* reason, bool* ok);

    uint64_t historyHandle_{0};
    QString documentJson_;
    QVector<RenderEntity> entities_;
    QString epsPolicyJson_;
    SelectionState selection_;
    QHash<QString, bool> layerVisible_;
    QHash<QString, bool> layerLocked_;
};
