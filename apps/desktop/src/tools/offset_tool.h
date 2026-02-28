#pragma once
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "tool_base.h"
#include <optional>

class OffsetTool : public ToolBase {
public:
    OffsetTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onPointerUp(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void renderOverlay(QPainter&) override;

private:
    bool commit(QString* reason);
    void updatePreview();

    DocStore* store_;
    Camera* camera_;
    QString targetId_;
    QJsonObject previewGeom_;
    std::optional<double> previewDist_;
    NumericInput numeric_;
};
