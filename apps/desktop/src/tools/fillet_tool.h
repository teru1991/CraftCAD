#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"
#include "ambiguity_controller.h"

class FilletTool : public ToolBase {
public:
    FilletTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void onWheel(QWheelEvent*) override;
    void renderOverlay(QPainter&) override;
private:
    DocStore* store_;
    Camera* camera_;
    QString a_, b_;
    NumericInput numeric_;
    AmbiguityController ambiguity_;
    QPointF last_;
};
