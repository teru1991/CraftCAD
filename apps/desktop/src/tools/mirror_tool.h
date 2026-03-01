#pragma once
#include "tool_base.h"
#include "../camera.h"
#include "../doc_store.h"

class MirrorTool : public ToolBase {
public:
    MirrorTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
private:
    DocStore* store_;
    Camera* camera_;
    bool haveA_{false};
    WVec2 a_, b_;
};
