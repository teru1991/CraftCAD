#pragma once
#include "../camera.h"
#include <QJsonArray>
#include <QPainter>
#include <QWheelEvent>

class AmbiguityController {
public:
    void setCandidates(const QJsonArray& cands) { candidates_ = cands; index_ = 0; }
    void clear() { candidates_ = QJsonArray{}; index_ = 0; }
    bool active() const { return !candidates_.isEmpty(); }
    int currentIndex() const { return index_; }
    void onWheel(QWheelEvent* e);
    void onTab(int delta);
    void render(QPainter& p, const Camera& camera) const;
private:
    QJsonArray candidates_;
    int index_{0};
};
