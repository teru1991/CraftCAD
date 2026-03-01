#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "../snap_engine.h"
#include <optional>

class CircleTool : public ToolBase {
public:
    CircleTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onPointerUp(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void renderOverlay(QPainter&) override;
private:
    DocStore* store_;
    Camera* camera_;
    std::optional<WVec2> c_;
    std::optional<WVec2> edge_;
    SnapResult snap_;
    NumericInput numeric_;
};
