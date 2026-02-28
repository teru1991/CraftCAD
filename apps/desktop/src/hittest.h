#pragma once
#include "camera.h"
#include "doc_store.h"
#include <optional>

struct Hit { QString entityId; WVec2 worldPoint; double dist{0.0}; QString kind; };

std::optional<Hit> hitTest(const DocStore& store, const Camera& camera, const QPointF& screenPos, double radiusPx);
