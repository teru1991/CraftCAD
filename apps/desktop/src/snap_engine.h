#pragma once
#include "camera.h"
#include "doc_store.h"
#include <optional>

struct SnapCandidate { WVec2 point; QString label; int rank; double dist; };
struct SnapResult { std::optional<SnapCandidate> best; QVector<SnapCandidate> all; };

SnapResult computeSnap(const DocStore& store, const WVec2& pointerWorld, const std::optional<WVec2>& lineStart);
