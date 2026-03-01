#pragma once
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "tool_base.h"
#include <QVector>
#include <optional>

class RotateTool : public ToolBase {
public:
    RotateTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onPointerUp(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void renderOverlay(QPainter&) override;

private:
    struct PreviewLine { WVec2 a; WVec2 b; };
    void rebuildCenter();
    void refreshPreview(const WVec2& current);
    bool commit(double angle, QString* reason);

    DocStore* store_;
    Camera* camera_;
    std::optional<WVec2> start_;
    std::optional<WVec2> center_;
    std::optional<WVec2> current_;
    QVector<PreviewLine> base_;
    QVector<PreviewLine> preview_;
    NumericInput numeric_;
    bool groupActive_{false};
};
