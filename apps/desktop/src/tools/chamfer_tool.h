#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"

class ChamferTool : public ToolBase {
public:
    ChamferTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
private:
    DocStore* store_;
    Camera* camera_;
    QString a_, b_;
    NumericInput numeric_;
};
