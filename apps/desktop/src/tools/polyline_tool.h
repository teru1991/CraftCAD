#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "../snap_engine.h"
#include <optional>

class PolylineTool : public ToolBase {
public:
    PolylineTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void renderOverlay(QPainter&) override;
private:
    void commit(bool closed);
    DocStore* store_;
    Camera* camera_;
    QVector<WVec2> pts_;
    std::optional<WVec2> hover_;
    SnapResult snap_;
    bool lockH_{false};
    bool lockV_{false};
    NumericInput numeric_;
};
