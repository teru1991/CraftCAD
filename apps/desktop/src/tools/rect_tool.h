#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "../snap_engine.h"
#include <optional>

class RectTool : public ToolBase {
public:
    RectTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onPointerUp(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void renderOverlay(QPainter&) override;
private:
    DocStore* store_;
    Camera* camera_;
    std::optional<WVec2> p0_;
    std::optional<WVec2> p1_;
    SnapResult snap_;
    bool lockH_{false};
    bool lockV_{false};
    NumericInput numeric_;
};
