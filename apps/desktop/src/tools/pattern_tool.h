#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"
#include "../input/numeric_input.h"

class PatternTool : public ToolBase {
public:
    PatternTool(DocStore* store, Camera* camera);
    void onKeyPress(QKeyEvent*) override;
private:
    DocStore* store_;
    Camera* camera_;
    NumericInput numeric_;
};
