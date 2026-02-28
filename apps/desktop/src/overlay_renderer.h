#pragma once
#include "camera.h"
#include "snap_engine.h"
#include <QPainter>

void renderOverlay(QPainter& p, const Camera& cam, const SnapResult& snap, const std::optional<QPair<WVec2,WVec2>>& previewLine);
