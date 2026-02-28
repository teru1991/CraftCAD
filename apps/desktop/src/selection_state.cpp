#include "selection_state.h"

void SelectionState::clear() { selected_.clear(); }
void SelectionState::setSingle(const QString& id) { selected_.clear(); selected_.insert(id); }
bool SelectionState::isSelected(const QString& id) const { return selected_.contains(id); }
const QSet<QString>& SelectionState::ids() const { return selected_; }
