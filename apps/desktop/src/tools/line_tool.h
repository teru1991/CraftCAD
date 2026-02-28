#pragma once
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "../snap_engine.h"
#include "tool_base.h"
#include <optional>

class LineTool : public ToolBase {
public:
    LineTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onPointerUp(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void renderOverlay(QPainter&) override;

private:
    enum class Constraint { None, Horizontal, Vertical };
    WVec2 applyConstraint(const WVec2& start, const WVec2& end) const;
    bool commitLine(const WVec2& start, const WVec2& end, QString* reason);

    DocStore* store_;
    Camera* camera_;
    std::optional<WVec2> start_;
    std::optional<WVec2> end_;
    SnapResult snap_;
    Constraint constraint_{Constraint::None};
    NumericInput numeric_;
    bool groupActive_{false};
};
