#pragma once
#include "../camera.h"
#include "../doc_store.h"
#include "tool_base.h"
#include <QJsonArray>

class TrimTool : public ToolBase {
public:
    TrimTool(DocStore* store, Camera* camera);
    void onPointerDown(const QPointF&) override;
    void onPointerMove(const QPointF&) override;
    void onPointerUp(const QPointF&) override;
    void onKeyPress(QKeyEvent*) override;
    void onWheel(QWheelEvent*) override;
    void renderOverlay(QPainter&) override;

private:
    enum class Step { PickTarget, PickCutter, PreviewOrAmbiguous };
    bool runTrim(QString* reason, bool commit, int candidateIndex = -1);
    void cycleCandidate(int delta);

    DocStore* store_;
    Camera* camera_;
    Step step_{Step::PickTarget};
    QString targetId_;
    QString cutterId_;
    QPointF lastScreen_;
    QJsonArray candidates_;
    int candidateIndex_{0};
};
